use crate::nicknamer::commands::names::{Names, NamesRepository};
use crate::nicknamer::commands::{Error, Reply, User};
use crate::nicknamer::config;
use crate::nicknamer::connectors::discord::server_member::ServerMember;
use crate::nicknamer::connectors::discord::{DiscordConnector, Role};
use log::info;

pub trait Revealer {
    async fn reveal_all(&self) -> Result<(), Error>;
    async fn reveal_member(&self, member: &ServerMember) -> Result<(), Error>;
}
pub struct RevealerImpl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> {
    names_repository: &'a REPO,
    discord_connector: &'a DISCORD,
}

impl<'a, NAMES: NamesRepository, DISCORD: DiscordConnector> RevealerImpl<'a, NAMES, DISCORD> {
    pub fn new(names_repository: &'a NAMES, discord_connector: &'a DISCORD) -> Self {
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
        info!("Revealing real name for {}", member.user_name);
        if member.is_bot {
            self.reveal_bot_member(member).await?;
        } else {
            self.reveal_human_member(member).await?;
        }
        Ok(())
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> RevealerImpl<'a, REPO, DISCORD> {
    async fn reveal_human_member(&self, member: &ServerMember) -> Result<(), Error> {
        let names = self.names_repository.load_real_names().await?;
        let reply = reveal_member(member, &names);
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> RevealerImpl<'a, REPO, DISCORD> {
    async fn reveal_bot_member(&self, member: &ServerMember) -> Result<(), Error> {
        let name_to_show = match &member.nick_name {
            Some(nick_name) => nick_name,
            None => &member.user_name,
        };
        let reply = format!("{} is a bot, {}!", name_to_show, config::REVEAL_INSULT);
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
        self.reveal_users_without_real_name(members, real_names)
            .await?;
        Ok(())
    }

    async fn reveal_users_with_real_name(
        &self,
        members: &[ServerMember],
        real_names: &Names,
    ) -> Result<(), Error> {
        let users: Vec<User> = get_users_with_real_names(&members, &real_names);
        info!("Found {} users with real names", users.len());
        if users.is_empty() {
            return Ok(());
        }
        let reply_for_users_with_real_name = create_reply_for_users_with_real_names(&users);
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
        if users.is_empty() {
            return Ok(());
        }
        let role_to_mention = self
            .discord_connector
            .get_role_by_name(config::CODE_MONKEYS_ROLE_NAME)
            .await?;
        let reply = create_reply_for_users_without_real_names(&users, &*role_to_mention);
        if reply.is_empty() {
            return Ok(());
        }
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

fn reveal_member(server_member: &ServerMember, real_names: &Names) -> Reply {
    let user_id = server_member.id;
    let mut user: User = server_member.into();
    let real_name = real_names.names.get(&user_id).cloned();
    user.real_name = real_name;
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
            let None = real_names.names.get(&member.id) else {
                return None;
            };
            if member.is_bot {
                return None;
            }
            let user: User = member.into();
            Some(user)
        })
        .collect()
}

fn create_reply_for_users_with_real_names(users: &[User]) -> Reply {
    assert!(
        !users.is_empty(),
        "You can't create a reply for an empty list of users"
    );
    let reply = users
        .into_iter()
        .map(|user| user.to_string())
        .collect::<Vec<String>>();

    format!(
        "Here are people's real names, {}:
\t{}",
        config::REVEAL_INSULT,
        reply.join("\n\t")
    )
}

fn create_reply_for_users_without_real_names(users: &[User], mention: &dyn Role) -> Reply {
    assert!(
        !users.is_empty(),
        "You can't create a reply for an empty list of users"
    );
    let reply = users
        .into_iter()
        .map(|user| user.to_string())
        .collect::<Vec<String>>();

    format!(
        "Hey {}, these members are unrecognized:
\t{}
One of y'all should improve real name management and/or add them to the config",
        mention.mention(),
        reply.join("\n\t")
    )
}

#[cfg(test)]
mod tests {
    // Tests for RevealerImpl using MockDiscordConnector and MockNamesRepository
    // Mock Role implementation for tests
    #[derive(Default)]
    struct MockRole {}

    impl MockRole {
        fn new() -> Self {
            Self::default()
        }
    }

    impl crate::nicknamer::connectors::discord::Mentionable for MockRole {
        fn mention(&self) -> String {
            "@CodeMonkeys".to_string()
        }
    }

    impl crate::nicknamer::connectors::discord::Role for MockRole {}

    mod revealer_impl_tests {
        use super::MockRole;
        use crate::nicknamer::commands::names::{MockNamesRepository, Names};
        use crate::nicknamer::commands::reveal::{Error, Revealer, RevealerImpl};
        use crate::nicknamer::config;
        use crate::nicknamer::connectors::discord::MockDiscordConnector;
        use crate::nicknamer::connectors::discord::server_member::ServerMemberBuilder;
        use mockall::predicate::*;
        use std::collections::HashMap;

        #[tokio::test]
        async fn can_successfully_reveal_all_members() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data
            let members = vec![
                ServerMemberBuilder::new()
                    .id(123456789)
                    .nick_name("AliceNickname")
                    .user_name("AliceUsername")
                    .is_bot(false)
                    .build(),
                ServerMemberBuilder::new()
                    .id(987654321)
                    .nick_name("BobNickname")
                    .user_name("BobUsername")
                    .is_bot(false)
                    .build(),
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
        async fn reveal_all_should_filter_out_bot_users() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data - one normal user and one bot user
            let members = vec![
                ServerMemberBuilder::new()
                    .id(111111111)
                    .nick_name("HumanUser")
                    .user_name("HumanUser")
                    .is_bot(false)
                    .build(),
                ServerMemberBuilder::new()
                    .id(222222222)
                    .nick_name("BotUser")
                    .user_name("BotUser")
                    .is_bot(true) // This is a bot
                    .build(),
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

            // Mock the role request
            mock_discord
                .expect_get_role_by_name()
                .with(eq("Code Monkeys"))
                .times(1)
                .returning(|_| Ok(Box::new(MockRole::new())));

            // Expect only one user (the human) in the message
            mock_discord
                .expect_send_reply()
                .with(always())
                .times(1)
                .returning(|message| {
                    // The bot user should not be included in the message
                    assert!(message.contains("HumanUser"));
                    assert!(!message.contains("BotUser"));
                    Ok(())
                });

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_all().await;

            // Verify results
            assert!(result.is_ok(), "reveal_all should succeed");
        }

        #[tokio::test]
        async fn reveal_all_with_all_bot_users_should_not_send_message() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data - only bot users
            let members = vec![
                ServerMemberBuilder::new()
                    .id(111111111)
                    .nick_name("BotUser1")
                    .user_name("BotUser1")
                    .is_bot(true)
                    .build(),
                ServerMemberBuilder::new()
                    .id(222222222)
                    .nick_name("BotUser2")
                    .user_name("BotUser2")
                    .is_bot(true)
                    .build(),
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

            // No message should be sent because all users are bots
            // So we don't expect any call to send_reply

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_all().await;

            // Verify results
            assert!(result.is_ok(), "reveal_all should succeed");
        }

        #[tokio::test]
        async fn reveal_member_should_handle_bot_with_nickname() {
            // Setup mock objects
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data - a bot user with nickname
            let bot_member = ServerMemberBuilder::new()
                .id(111111111)
                .nick_name("BotNick")
                .user_name("BotUser")
                .is_bot(true)
                .build();

            // Check that the correct message is sent via Discord
            mock_discord
                .expect_send_reply()
                .with(eq(format!("BotNick is a bot, {}!", config::REVEAL_INSULT)))
                .times(1)
                .returning(|_| Ok(()));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_member(&bot_member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal_member should succeed for a bot with nickname"
            );
        }

        #[tokio::test]
        async fn reveal_member_should_handle_bot_without_nickname() {
            // Setup mock objects
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data - a bot user without nickname
            let bot_member = ServerMemberBuilder::new()
                .id(222222222)
                .user_name("BotUserName")
                .is_bot(true)
                .build();

            // Check that the correct message is sent via Discord
            mock_discord
                .expect_send_reply()
                .with(eq(format!(
                    "BotUserName is a bot, {}!",
                    config::REVEAL_INSULT
                )))
                .times(1)
                .returning(|_| Ok(()));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_member(&bot_member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal_member should succeed for a bot without nickname"
            );
        }

        #[tokio::test]
        async fn reveal_member_should_handle_discord_error_for_bot() {
            // Setup mock objects
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data - a bot user
            let bot_member = ServerMemberBuilder::new()
                .id(333333333)
                .nick_name("ErrorBot")
                .user_name("ErrorBot")
                .is_bot(true)
                .build();

            // Discord connector returns an error when trying to send reply
            mock_discord
                .expect_send_reply()
                .times(1)
                .returning(|_| Err(crate::nicknamer::connectors::discord::Error::CannotSendReply));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_member(&bot_member).await;

            // Verify results
            assert!(
                result.is_err(),
                "reveal_member should fail when Discord errors"
            );
            match result {
                Err(Error::DiscordError(_)) => (),
                _ => panic!("Expected DiscordError, got different error type"),
            }
        }

        #[tokio::test]
        async fn reveal_member_should_handle_member_without_real_name() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data - a user without a real name in the database
            let member = ServerMemberBuilder::new()
                .id(111111111)
                .nick_name("NickName")
                .user_name("UserName")
                .is_bot(false)
                .build();

            // Empty names database
            let names = Names {
                names: HashMap::new(),
            };

            // Set up expectations
            mock_repo
                .expect_load_real_names()
                .times(1)
                .returning(move || Ok(names.clone()));

            // The message should show username and nickname but no real name
            mock_discord
                .expect_send_reply()
                .with(eq("UserName aka 'NickName'"))
                .times(1)
                .returning(|_| Ok(()));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_member(&member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal_member should succeed for a member without real name"
            );
        }

        #[tokio::test]
        async fn reveal_member_should_handle_member_with_real_name() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();

            // Define test data - a user with a real name in the database
            let member = ServerMemberBuilder::new()
                .id(111111111)
                .nick_name("NickName")
                .user_name("UserName")
                .is_bot(false)
                .build();

            // Names database with the user's real name
            let mut names_map = HashMap::new();
            names_map.insert(111111111, "Real Person".to_string());
            let names = Names { names: names_map };

            // Set up expectations
            mock_repo
                .expect_load_real_names()
                .times(1)
                .returning(move || Ok(names.clone()));

            // The message should include the real name
            mock_discord
                .expect_send_reply()
                .with(eq("'NickName' is Real Person"))
                .times(1)
                .returning(|_| Ok(()));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_member(&member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal_member should succeed for a member with real name"
            );
        }

        #[tokio::test]
        async fn reveal_member_should_handle_names_repository_error() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mock_discord = MockDiscordConnector::new();

            // Define test data
            let member = ServerMemberBuilder::new()
                .id(111111111)
                .nick_name("NickName")
                .user_name("UserName")
                .is_bot(false)
                .build();

            // Set up expectations - repository returns an error
            mock_repo
                .expect_load_real_names()
                .times(1)
                .returning(|| Err(crate::nicknamer::commands::names::Error::CannotLoadNames));

            // Create revealer with mock objects
            let revealer = RevealerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = revealer.reveal_member(&member).await;

            // Verify results
            assert!(
                result.is_err(),
                "reveal_member should fail when repository fails"
            );
            assert!(
                matches!(result.unwrap_err(), Error::NamesAccessError(_)),
                "Error should be a NamesAccessError"
            );
        }
    }
}
