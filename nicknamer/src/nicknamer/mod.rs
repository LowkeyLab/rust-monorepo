pub(crate) mod commands;
mod config;

pub(crate) mod connectors;

use crate::nicknamer::connectors::discord;
use commands::Error;
use commands::names::NamesRepository;
use commands::reveal::{Revealer, RevealerImpl};
use connectors::discord::DiscordConnector;

pub trait Nicknamer {
    async fn reveal_all(&self) -> Result<(), Error>;
    async fn reveal(&self, member: &discord::ServerMember) -> Result<(), Error>;
}

pub struct NicknamerImpl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> {
    names_repository: &'a REPO,
    discord_connector: &'a DISCORD,
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> NicknamerImpl<'a, REPO, DISCORD> {
    pub fn new(names_repository: &'a REPO, discord_connector: &'a DISCORD) -> Self {
        Self {
            names_repository,
            discord_connector,
        }
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> Nicknamer
    for NicknamerImpl<'a, REPO, DISCORD>
{
    async fn reveal_all(&self) -> Result<(), Error> {
        let revealer = RevealerImpl::new(self.names_repository, self.discord_connector);
        revealer.reveal_all().await
    }

    async fn reveal(&self, member: &discord::ServerMember) -> Result<(), Error> {
        let revealer = RevealerImpl::new(self.names_repository, self.discord_connector);
        revealer.reveal_member(member).await
    }
}

#[cfg(test)]
mod reveal_tests {
    // Tests for NicknamerImpl using MockDiscordConnector and MockNamesRepository
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

    mod nicknamer_impl_tests {
        use super::MockRole;
        use crate::nicknamer::commands::Error;
        use crate::nicknamer::commands::names::{MockNamesRepository, Names};
        use crate::nicknamer::config;
        use crate::nicknamer::connectors::discord::MockDiscordConnector;
        use crate::nicknamer::connectors::discord::server_member::ServerMemberBuilder;
        use crate::nicknamer::{Nicknamer, NicknamerImpl};
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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal_all().await;

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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal_all().await;

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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal_all().await;

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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal_all().await;

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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal(&bot_member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal should succeed for a bot with nickname"
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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal(&bot_member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal should succeed for a bot without nickname"
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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal(&bot_member).await;

            // Verify results
            assert!(result.is_err(), "reveal should fail when Discord errors");
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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal(&member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal should succeed for a member without real name"
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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal(&member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal should succeed for a member with real name"
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

            // Create nicknamer with mock objects
            let nicknamer = NicknamerImpl::new(&mock_repo, &mock_discord);

            // Execute the method under test
            let result = nicknamer.reveal(&member).await;

            // Verify results
            assert!(result.is_err(), "reveal should fail when repository fails");
            assert!(
                matches!(result.unwrap_err(), Error::NamesAccessError(_)),
                "Error should be a NamesAccessError"
            );
        }
    }
}
