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
        info!("Revealing nicknames for current channel members ...");
        let members = self
            .discord_connector
            .get_members_of_current_channel()
            .await?;
        let real_names = self.names_repository.load_real_names().await?;
        let reply = reveal_all_members(&members, &real_names)?;
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }

    async fn reveal_member(&self, member: &ServerMember) -> Result<(), Error> {
        info!("Revealing nickname for {}", member.user_name);
        let names = self.names_repository.load_real_names().await?;
        let reply = reveal_member(member, &names)?;
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

pub fn reveal_member(server_member: &ServerMember, real_names: &Names) -> Result<Reply, Error> {
    let user_id = server_member.id;
    let mut user: User = server_member.into();
    let real_name = real_names.names.get(&user_id).cloned();
    user.real_name = real_name;
    create_reply_for(&user)
}

fn reveal_all_members(members: &[ServerMember], real_names: &Names) -> Result<Reply, Error> {
    let users: Vec<User> = members
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
        .collect();
    info!("Found {} users with real names", users.len());
    create_reply_for_all(&users)
}

fn create_reply_for(user: &User) -> Result<Reply, Error> {
    match user.real_name {
        Some(_) => Ok(user.to_string()),
        None => Ok(format!(
            "How mysterious! {}'s true name is shrouded by darkness",
            user.display_name
        )),
    }
}

fn create_reply_for_all(users: &[User]) -> Result<Reply, Error> {
    let users = users
        .into_iter()
        .filter(|user| user.real_name.is_some())
        .filter_map(|user| match create_reply_for(user) {
            Ok(reply) => Some(reply),
            Err(err) => {
                info!("Error creating reply for user: {}", err);
                None
            }
        })
        .collect::<Vec<String>>();
    if users.is_empty() {
        return Ok("Y'all a bunch of unimportant, good fer nothing no-names".to_string());
    }

    Ok(format!(
        "Here are people's real names, {}:
{}",
        config::REVEAL_INSULT,
        users.join("\n")
    ))
}

#[cfg(test)]
mod tests {
    use crate::nicknamer::commands::names::Names;
    use crate::nicknamer::commands::reveal;
    use crate::nicknamer::commands::*;
    use crate::nicknamer::config;
    use crate::nicknamer::connectors::discord::ServerMember;
    use std::collections::HashMap;

    fn create_test_real_names() -> Names {
        let mut names = HashMap::new();
        names.insert(123456789, "Alice".to_string());
        names.insert(987654321, "Bob".to_string());
        Names { names }
    }

    fn create_server_member(id: u64, nickname: Option<String>, username: String) -> ServerMember {
        ServerMember {
            id,
            nick_name: nickname,
            user_name: username,
        }
    }

    #[test]
    fn test_reveal_member_with_real_name() {
        // Setup
        let real_names = create_test_real_names();
        let server_member = create_server_member(
            123456789,
            Some("AliceNickname".to_string()),
            "AliceUsername".to_string(),
        );

        // Call the function
        let result = reveal::reveal_member(&server_member, &real_names).unwrap();

        // The result should contain the user's nickname (or username if no nickname) and real name
        assert_eq!(result, "'AliceNickname' is Alice");
    }

    #[test]
    fn test_reveal_member_without_nickname() {
        // Setup - member with username but no nickname
        let real_names = create_test_real_names();
        let server_member = create_server_member(123456789, None, "AliceUsername".to_string());

        // Call the function
        let result = reveal::reveal_member(&server_member, &real_names).unwrap();

        // Should use the username when no nickname is available
        assert_eq!(result, "'AliceUsername' is Alice");
    }

    #[test]
    fn test_reveal_member_without_real_name() {
        // Setup - member with an ID that doesn't exist in real_names
        let real_names = create_test_real_names();
        let server_member = create_server_member(
            111222333,
            Some("UnknownNickname".to_string()),
            "UnknownUsername".to_string(),
        );

        // Call the function
        let result = reveal::reveal_member(&server_member, &real_names).unwrap();

        // Should return the "mysterious" message for users without real names
        assert_eq!(
            result,
            "How mysterious! UnknownNickname's true name is shrouded by darkness"
        );
    }

    #[test]
    fn test_reveal_member_with_special_characters() {
        // Setup
        let mut real_names = create_test_real_names();
        real_names
            .names
            .insert(555666777, "User with \"quotes\"".to_string());
        let server_member = create_server_member(
            555666777,
            Some("Nickname'With'Apostrophes".to_string()),
            "Username".to_string(),
        );

        // Call the function
        let result = reveal::reveal_member(&server_member, &real_names).unwrap();

        // Should handle special characters correctly
        assert_eq!(
            result,
            "'Nickname'With'Apostrophes' is User with \"quotes\""
        );
    }

    #[test]
    fn test_reveal_member_preserves_nickname_case() {
        // Setup
        let real_names = create_test_real_names();
        let server_member = create_server_member(
            123456789,
            Some("ALICE_UPPERCASE".to_string()),
            "alice_lowercase".to_string(),
        );

        // Call the function
        let result = reveal::reveal_member(&server_member, &real_names).unwrap();

        // Should preserve the nickname case exactly
        assert_eq!(result, "'ALICE_UPPERCASE' is Alice");
    }

    #[test]
    fn test_multiple_members_with_same_real_name() {
        // Setup - two members with the same real name
        let mut real_names = create_test_real_names();
        real_names.names.insert(111222333, "Bob".to_string());

        // First member
        let server_member1 = create_server_member(
            987654321,
            Some("BobNickname1".to_string()),
            "BobUsername1".to_string(),
        );
        let result1 = reveal::reveal_member(&server_member1, &real_names).unwrap();

        // Second member
        let server_member2 = create_server_member(
            111222333,
            Some("BobNickname2".to_string()),
            "BobUsername2".to_string(),
        );
        let result2 = reveal::reveal_member(&server_member2, &real_names).unwrap();

        // Different nicknames but same real name
        assert_eq!(result1, "'BobNickname1' is Bob");
        assert_eq!(result2, "'BobNickname2' is Bob");
    }

    #[test]
    fn test_conversion_from_server_member_to_user() {
        // Setup
        let server_member = create_server_member(
            123456789,
            Some("Nickname".to_string()),
            "Username".to_string(),
        );

        // Convert to User manually as reveal_member would do
        let user: User = server_member.into();

        // Verify conversion worked as expected
        assert_eq!(user.id, 123456789);
        assert_eq!(user.display_name, "Nickname");
        assert_eq!(user.real_name, None); // real_name should be None initially
    }

    #[test]
    fn revealing_user_with_no_nickname_results_in_insult() {
        // Test for a user with no real name
        let result = reveal::create_reply_for(&User {
            id: 0,
            display_name: "Unknown User".to_string(),
            real_name: None,
        })
        .unwrap();

        // The Display implementation for User should return an empty string when real_name is None
        assert_eq!(
            result,
            "How mysterious! Unknown User's true name is shrouded by darkness"
        );
    }

    #[test]
    fn test_reveal_all_members_with_real_names() {
        // Setup
        let real_names = create_test_real_names();
        let members = vec![
            create_server_member(
                123456789,
                Some("AliceNickname".to_string()),
                "AliceUsername".to_string(),
            ),
            create_server_member(
                987654321,
                Some("BobNickname".to_string()),
                "BobUsername".to_string(),
            ),
        ];

        // Call the function
        let result = reveal::reveal_all_members(&members, &real_names).unwrap();

        // The result should contain all users with real names
        let expected_header = format!("Here are people's real names, {}:", config::REVEAL_INSULT);
        assert!(
            result.starts_with(&expected_header),
            "Result should start with the expected header"
        );
        assert!(
            result.contains("'AliceNickname' is Alice"),
            "Result should contain Alice's information"
        );
        assert!(
            result.contains("'BobNickname' is Bob"),
            "Result should contain Bob's information"
        );
    }

    #[test]
    fn test_reveal_all_members_with_some_without_real_names() {
        // Setup
        let real_names = create_test_real_names();
        let members = vec![
            create_server_member(
                123456789,
                Some("AliceNickname".to_string()),
                "AliceUsername".to_string(),
            ),
            create_server_member(
                111222333, // ID not in real_names
                Some("UnknownUser".to_string()),
                "UnknownUsername".to_string(),
            ),
            create_server_member(
                987654321,
                Some("BobNickname".to_string()),
                "BobUsername".to_string(),
            ),
        ];

        // Call the function
        let result = reveal::reveal_all_members(&members, &real_names).unwrap();

        // The result should only contain users with real names (Alice and Bob)
        let expected_header = format!("Here are people's real names, {}:", config::REVEAL_INSULT);
        assert!(
            result.starts_with(&expected_header),
            "Result should start with the expected header"
        );
        assert!(
            result.contains("'AliceNickname' is Alice"),
            "Result should contain Alice's information"
        );
        assert!(
            result.contains("'BobNickname' is Bob"),
            "Result should contain Bob's information"
        );
        assert!(
            !result.contains("UnknownUser"),
            "Result should not contain unknown user's information"
        );
    }

    #[test]
    fn test_reveal_all_members_with_no_real_names() {
        // Setup
        let real_names = create_test_real_names();
        let members = vec![
            create_server_member(
                111222333, // ID not in real_names
                Some("UnknownUser1".to_string()),
                "UnknownUsername1".to_string(),
            ),
            create_server_member(
                444555666, // ID not in real_names
                Some("UnknownUser2".to_string()),
                "UnknownUsername2".to_string(),
            ),
        ];

        // Call the function
        let result = reveal::reveal_all_members(&members, &real_names).unwrap();

        // For no real names, we should get the "unimportant" message
        assert_eq!(
            result,
            "Y'all a bunch of unimportant, good fer nothing no-names"
        );
    }

    #[test]
    fn test_reveal_all_members_with_empty_members_list() {
        // Setup
        let real_names = create_test_real_names();
        let members: Vec<ServerMember> = vec![];

        // Call the function
        let result = reveal::reveal_all_members(&members, &real_names).unwrap();

        // For empty members list, we should also get the "unimportant" message
        assert_eq!(
            result,
            "Y'all a bunch of unimportant, good fer nothing no-names"
        );
    }

    // Tests for RevealerImpl using MockDiscordConnector and MockNamesRepository
    mod revealer_impl_tests {
        use super::*;
        use crate::nicknamer::commands::names::{MockNamesRepository, Names};
        use crate::nicknamer::commands::reveal::{Revealer, RevealerImpl};
        use crate::nicknamer::connectors::discord::{MockDiscordConnector, ServerMember};
        use mockall::predicate::*;
        use std::collections::HashMap;

        #[tokio::test]
        async fn test_reveal_all_success() {
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
        async fn test_reveal_all_discord_error() {
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
                matches!(result.unwrap_err(), reveal::Error::DiscordError(_)),
                "Error should be a DiscordError"
            );
        }

        #[tokio::test]
        async fn test_reveal_all_names_error() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data
            let members = vec![ServerMember {
                id: 123456789,
                nick_name: Some("AliceNickname".to_string()),
                user_name: "AliceUsername".to_string(),
            }];

            // Set up expectations
            mock_discord
                .expect_get_members_of_current_channel()
                .times(1)
                .returning(move || Ok(members.clone()));

            mock_repo
                .expect_load_real_names()
                .times(1)
                .returning(move || Err(names::Error::CannotLoadNames));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_all().await;

            // Verify results
            assert!(result.is_err(), "reveal_all should fail");
            assert!(
                matches!(result.unwrap_err(), reveal::Error::NamesAccessError(_)),
                "Error should be a NamesAccessError"
            );
        }

        #[tokio::test]
        async fn test_reveal_member_success() {
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
        async fn test_reveal_member_without_real_name() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data
            let member = ServerMember {
                id: 123456789,
                nick_name: Some("AliceNickname".to_string()),
                user_name: "AliceUsername".to_string(),
            };

            // Create an empty names map (no real name for Alice)
            let names = Names {
                names: HashMap::new(),
            };

            // Set up expectations
            mock_repo
                .expect_load_real_names()
                .times(1)
                .returning(move || Ok(names.clone()));

            mock_discord
                .expect_send_reply()
                .with(eq(
                    "How mysterious! AliceNickname's true name is shrouded by darkness",
                ))
                .times(1)
                .returning(|_| Ok(()));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_member(&member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal_member should succeed even when no real name is found"
            );
        }

        #[tokio::test]
        async fn test_reveal_member_discord_send_error() {
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
                matches!(result.unwrap_err(), reveal::Error::DiscordError(_)),
                "Error should be a DiscordError"
            );
        }
    }
}
