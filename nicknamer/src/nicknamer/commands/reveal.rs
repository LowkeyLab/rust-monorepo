use crate::nicknamer::commands::names::{Names, NamesRepository};
use crate::nicknamer::commands::{Reply, User, names};
use crate::nicknamer::config;
use crate::nicknamer::connectors::discord;
use crate::nicknamer::connectors::discord::{DiscordConnector, ServerMember};
use log::info;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Something went wrong with Discord")]
    DiscordError(#[from] discord::Error),
    #[error("Something went wrong getting people's names")]
    NamesAccessError(#[from] names::Error),
}
pub trait Revealer {
    async fn reveal_all(&self) -> Result<(), Error>;
    async fn reveal_member(&self, member: &ServerMember) -> Result<(), Error>;
}
pub struct RevealerImpl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> {
    names_repository: &'a REPO,
    discord_connector: &'a DISCORD,
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> RevealerImpl<'a, REPO, DISCORD> {
    pub fn new(names_repository: &'a REPO, discord_connector: &'a DISCORD) -> Self {
        Self {
            names_repository,
            discord_connector,
        }
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> Revealer
    for RevealerImpl<'a, REPO, DISCORD>
{
    async fn reveal_all(&self) -> Result<(), Error> {
        info!("Revealing real names for current channel members ...");
        let members = self
            .discord_connector
            .get_members_of_current_channel()
            .await?;
        let real_names = self.names_repository.load_real_names().await?;
        self.reveal_all_members(&members, &real_names).await?;
        Ok(())
    }

    async fn reveal_member(&self, member: &ServerMember) -> Result<(), Error> {
        info!("Revealing nickname for {}", member.user_name);
        let names = self.names_repository.load_real_names().await?;
        let reply = reveal_member(member, &names);
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> RevealerImpl<'a, REPO, DISCORD> {
    async fn reveal_all_members(
        &self,
        members: &[ServerMember],
        real_names: &Names,
    ) -> Result<(), Error> {
        self.reveal_users_with_real_name(members, real_names)
            .await?;
        // self.reveal_users_without_real_name(members, real_names)
        //     .await?;
        Ok(())
    }

    async fn reveal_users_with_real_name(
        &self,
        members: &[ServerMember],
        real_names: &Names,
    ) -> Result<(), Error> {
        let users_with_real_names: Vec<User> = get_users_with_real_names(&members, &real_names);
        info!(
            "Found {} users with real names",
            users_with_real_names.len()
        );
        let reply_for_users_with_real_name =
            create_reply_for_users_with_real_names(&users_with_real_names)?;
        self.discord_connector
            .send_reply(&reply_for_users_with_real_name)
            .await?;
        Ok(())
    }

    async fn reveal_users_without_real_name(
        &self,
        members: &[ServerMember],
        real_names: &Names,
    ) -> Result<(), Error> {
        let users: Vec<User> = get_users_without_real_names(members, real_names);
        info!("Found {} users without real names", users.len());
        let reply = create_reply_for_users_without_real_names(&users)?;
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

fn reveal_member(server_member: &ServerMember, real_names: &Names) -> Reply {
    let user_id = server_member.id;
    let mut user: User = server_member.into();
    let real_name = real_names.names.get(&user_id).cloned();
    user.real_name = real_name;
    assert!(
        user.real_name.is_some(),
        "You can't create a reply for a user without a real name"
    );
    user.to_string()
}

fn get_users_with_real_names(members: &[ServerMember], real_names: &Names) -> Vec<User> {
    members
        .iter()
        .filter_map(|member| {
            // Only include users with real names in our database
            let Some(real_name) = real_names.names.get(&member.id) else {
                return None;
            };
            let mut user: User = member.into();
            user.real_name = Some(real_name.clone());
            Some(user)
        })
        .collect()
}

fn get_users_without_real_names(members: &[ServerMember], real_names: &Names) -> Vec<User> {
    members
        .iter()
        .filter_map(|member| {
            // Only include users with real names in our database
            let None = real_names.names.get(&member.id) else {
                return None;
            };
            let user: User = member.into();
            Some(user)
        })
        .collect()
}

fn create_reply_for_users_with_real_names(users: &[User]) -> Result<Reply, Error> {
    if users.is_empty() {
        return Ok("Y'all a bunch of unimportant, good fer nothing no-names".to_string());
    }

    let reply_for_users_with_real_names = users
        .into_iter()
        .map(|user| user.to_string())
        .collect::<Vec<String>>();

    Ok(format!(
        "Here are people's real names, {}:
{}",
        config::REVEAL_INSULT,
        reply_for_users_with_real_names.join("\n")
    ))
}

fn create_reply_for_users_without_real_names(users: &[User]) -> Result<Reply, Error> {
    todo!()
}

#[cfg(test)]
mod tests {
    // Tests for RevealerImpl using MockDiscordConnector and MockNamesRepository
    mod revealer_impl_tests {
        use crate::nicknamer::commands::names::{MockNamesRepository, Names};
        use crate::nicknamer::commands::reveal::{Error, Revealer, RevealerImpl};
        use crate::nicknamer::connectors::discord::{MockDiscordConnector, ServerMember};
        use mockall::predicate::*;
        use std::collections::HashMap;

        #[tokio::test]
        async fn can_successfully_reveal_all_members() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data
            let members = vec![
                ServerMember {
                    id: 123456789,
                    nick_name: Some("AliceNickname".to_string()),
                    user_name: "AliceUsername".to_string(),
                },
                ServerMember {
                    id: 987654321,
                    nick_name: Some("BobNickname".to_string()),
                    user_name: "BobUsername".to_string(),
                },
            ];

            let mut names_map = HashMap::new();
            names_map.insert(123456789, "Alice".to_string());
            names_map.insert(987654321, "Bob".to_string());
            let names = Names { names: names_map };

            // Set up expectations
            mock_discord
                .expect_get_members_of_current_channel()
                .times(1)
                .returning(move || Ok(members.clone()));

            mock_repo
                .expect_load_real_names()
                .times(1)
                .returning(move || Ok(names.clone()));

            mock_discord
                .expect_send_reply()
                .with(always()) // We don't test exact message content here as that's tested separately
                .times(1)
                .returning(|_| Ok(()));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_all().await;

            // Verify results
            assert!(result.is_ok(), "reveal_all should succeed");
        }

        #[tokio::test]
        async fn handles_discord_error_when_revealing_all() {
            // Setup mock objects
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Setup discord connector to return an error
            mock_discord
                .expect_get_members_of_current_channel()
                .times(1)
                .returning(|| {
                    Err(crate::nicknamer::connectors::discord::Error::NotInServerChannel)
                });

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_all().await;

            // Verify results
            assert!(result.is_err(), "reveal_all should fail");
            assert!(
                matches!(result.unwrap_err(), Error::DiscordError(_)),
                "Error should be a DiscordError"
            );
        }

        #[tokio::test]
        async fn can_successfully_reveal_single_member() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data
            let member = ServerMember {
                id: 123456789,
                nick_name: Some("AliceNickname".to_string()),
                user_name: "AliceUsername".to_string(),
            };

            let mut names_map = HashMap::new();
            names_map.insert(123456789, "Alice".to_string());
            let names = Names { names: names_map };

            // Set up expectations
            mock_repo
                .expect_load_real_names()
                .times(1)
                .returning(move || Ok(names.clone()));

            mock_discord
                .expect_send_reply()
                .with(eq("'AliceNickname' is Alice"))
                .times(1)
                .returning(|_| Ok(()));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_member(&member).await;

            // Verify results
            assert!(result.is_ok(), "reveal_member should succeed");
        }

        #[tokio::test]
        async fn handles_discord_error_when_revealing_single_member() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data
            let member = ServerMember {
                id: 123456789,
                nick_name: Some("AliceNickname".to_string()),
                user_name: "AliceUsername".to_string(),
            };

            let mut names_map = HashMap::new();
            names_map.insert(123456789, "Alice".to_string());
            let names = Names { names: names_map };

            // Set up expectations
            mock_repo
                .expect_load_real_names()
                .times(1)
                .returning(move || Ok(names.clone()));

            mock_discord
                .expect_send_reply()
                .times(1)
                .returning(|_| Err(crate::nicknamer::connectors::discord::Error::CannotSendReply));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_member(&member).await;

            // Verify results
            assert!(result.is_err(), "reveal_member should fail");
            assert!(
                matches!(result.unwrap_err(), Error::DiscordError(_)),
                "Error should be a DiscordError"
            );
        }

        #[tokio::test]
        async fn displays_correct_message_when_no_members_have_real_names() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data - channel members with IDs not in the real names database
            let members = vec![
                ServerMember {
                    id: 111111111, // ID not in real_names database
                    nick_name: Some("UnknownNick1".to_string()),
                    user_name: "UnknownUser1".to_string(),
                },
                ServerMember {
                    id: 222222222, // ID not in real_names database
                    nick_name: Some("UnknownNick2".to_string()),
                    user_name: "UnknownUser2".to_string(),
                },
            ];

            // Empty real names database
            let names = Names {
                names: HashMap::new(),
            };

            // Set up expectations
            mock_discord
                .expect_get_members_of_current_channel()
                .times(1)
                .returning(move || Ok(members.clone()));

            mock_repo
                .expect_load_real_names()
                .times(1)
                .returning(move || Ok(names.clone()));

            // Expect the correct message to be sent
            mock_discord
                .expect_send_reply()
                .with(eq(
                    "Y'all a bunch of unimportant, good fer nothing no-names",
                ))
                .times(1)
                .returning(|_| Ok(()));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_all().await;

            // Verify results
            assert!(result.is_ok(), "reveal_all should succeed");
        }
    }
}
