pub mod config;

pub(crate) mod connectors;
pub(crate) mod names;
pub(crate) mod user;

use crate::nicknamer::config::NicknamerConfig;
use crate::nicknamer::connectors::discord;
use async_trait::async_trait;
use connectors::discord::DiscordConnector;
use names::NamesRepository;
use tracing::info;
use user::Error;
use user::User;

#[async_trait]
pub trait Nicknamer {
    async fn reveal_all(&self) -> Result<(), Error>;
    async fn reveal(&self, member: &discord::ServerMember) -> Result<(), Error>;
    async fn change_nickname(
        &self,
        member: &discord::ServerMember,
        new_nickname: &str,
    ) -> Result<(), Error>;
}

pub struct NicknamerImpl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> {
    names_repository: &'a REPO,
    discord_connector: &'a DISCORD,
    config: &'a NicknamerConfig,
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> NicknamerImpl<'a, REPO, DISCORD> {
    pub fn new(
        names_repository: &'a REPO,
        discord_connector: &'a DISCORD,
        config: &'a NicknamerConfig,
    ) -> Self {
        Self {
            names_repository,
            discord_connector,
            config,
        }
    }

    async fn admonish_for_violating_party_guidelines(&self) -> Result<(), Error> {
        let reply = "You dare to rename our great General Secretary??? Away with your impudence!";
        self.discord_connector.send_reply(reply).await?;
        Ok(())
    }

    async fn change_member_nick_name(
        &self,
        member: &discord::ServerMember,
        new_nick_name: &str,
    ) -> Result<(), Error> {
        match self
            .discord_connector
            .change_member_nick_name(member.id, new_nick_name)
            .await
        {
            Ok(()) => match &member.nick_name {
                Some(nick_name) => {
                    self.send_reply_for_member_with_nick_name(member, new_nick_name, nick_name)
                        .await?
                }
                None => {
                    self.send_reply_for_member_without_nick_name(member, new_nick_name)
                        .await?
                }
            },
            Err(err) => {
                let reply = match err {
                    discord::Error::NotEnoughPermissions => {
                        let role_to_mention = self
                            .discord_connector
                            .get_role_by_name(&self.config.reveal.role_to_mention)
                            .await?;
                        format!(
                            "Some devilry restricts my power. {} please investigate the rogue member {}",
                            role_to_mention.mention(),
                            member.mention
                        )
                    }
                    err => format!("You fool! You messed it up!: {}", err),
                };
                self.discord_connector.send_reply(&reply).await?;
            }
        }
        Ok(())
    }

    async fn send_reply_for_member_without_nick_name(
        &self,
        member: &discord::ServerMember,
        new_nick_name: &str,
    ) -> Result<(), Error> {
        let reply = format!(
            "{} has been christened with the name {}!",
            member.user_name, new_nick_name
        );
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }

    async fn send_reply_for_member_with_nick_name(
        &self,
        member: &discord::ServerMember,
        new_nick_name: &str,
        nick_name: &String,
    ) -> Result<(), Error> {
        let reply = format!(
            "Changed {}'s nickname from '{}' to '{}'",
            member.user_name, nick_name, new_nick_name
        );
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

#[async_trait]
impl<REPO: NamesRepository + Send + Sync, DISCORD: DiscordConnector + Send + Sync> Nicknamer
    for NicknamerImpl<'_, REPO, DISCORD>
{
    async fn reveal_all(&self) -> Result<(), Error> {
        info!("Revealing real names for current channel members ...");
        let members = self
            .discord_connector
            .get_members_of_current_channel()
            .await?;
        let real_names = self.names_repository.load_real_names().await?;

        // Reveal users with real names
        let users_with_real_names: Vec<User> = members
            .iter()
            .filter_map(|member| {
                // Only include users with real names in our database
                let real_name = real_names.names.get(&member.id)?;
                let mut user: User = member.into();
                user.real_name = Some(real_name.clone());
                Some(user)
            })
            .collect();

        info!(
            "Found {} users with real names",
            users_with_real_names.len()
        );
        if !users_with_real_names.is_empty() {
            let reply = users_with_real_names
                .iter()
                .map(|user| Self::format_user(user))
                .collect::<Vec<String>>();

            let formatted_reply = format!(
                "Here are people's real names, {}:
\t{}",
                self.config.reveal.insult,
                reply.join("\n\t")
            );

            self.discord_connector.send_reply(&formatted_reply).await?;
        }

        // Reveal users without real names
        let users_without_real_names: Vec<User> = members
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
            .collect();

        info!(
            "Found {} users without real names",
            users_without_real_names.len()
        );
        if !users_without_real_names.is_empty() {
            let role_to_mention = self
                .discord_connector
                .get_role_by_name(&self.config.reveal.role_to_mention)
                .await?;

            let reply = users_without_real_names
                .iter()
                .map(|user| Self::format_user(user))
                .collect::<Vec<String>>();

            let formatted_reply = format!(
                "Hey {}, these members are unrecognized:
                \t{}
                One of y'all should improve real name management and/or add them to the config",
                role_to_mention.mention(),
                reply.join("\n\t")
            );

            if !formatted_reply.is_empty() {
                self.discord_connector.send_reply(&formatted_reply).await?;
            }
        }

        Ok(())
    }

    async fn reveal(&self, member: &discord::ServerMember) -> Result<(), Error> {
        info!("Revealing real name for {}", member.user_name);
        if member.is_bot {
            // Handle bot member
            let name_to_show = match &member.nick_name {
                Some(nick_name) => nick_name,
                None => &member.user_name,
            };
            let reply = format!("{} is a bot, {}!", name_to_show, &self.config.reveal.insult);
            self.discord_connector.send_reply(&reply).await?;
        } else {
            // Handle human member
            let names = self.names_repository.load_real_names().await?;
            let user_id = member.id;
            let mut user: User = member.into();
            let real_name = names.names.get(&user_id).cloned();
            user.real_name = real_name;
            let reply = Self::format_user(&user);
            self.discord_connector.send_reply(&reply).await?;
        }
        Ok(())
    }

    async fn change_nickname(
        &self,
        member: &discord::ServerMember,
        new_nickname: &str,
    ) -> Result<(), Error> {
        let owner_id = self.discord_connector.get_guild_owner_id().await?;
        if member.id == owner_id {
            self.admonish_for_violating_party_guidelines().await?;
        } else {
            self.change_member_nick_name(member, new_nickname).await?;
        }
        Ok(())
    }
}

impl<REPO: NamesRepository + Send + Sync, DISCORD: DiscordConnector + Send + Sync>
    NicknamerImpl<'_, REPO, DISCORD>
{
    fn format_user(user: &User) -> String {
        if let Some(real_name) = &user.real_name {
            if let Some(nick_name) = &user.nick_name {
                format!("'{}' is {}", nick_name, real_name)
            } else {
                format!("'{}' is {}", user.user_name, real_name)
            }
        } else if let Some(nick_name) = &user.nick_name {
            format!("{} aka '{}'", user.user_name, nick_name)
        } else {
            format!("{} has neither a nickname nor a real name", user.user_name)
        }
    }
}

#[cfg(test)]
mod nicknamer_impl_tests {
    use crate::nicknamer::NicknamerImpl;
    use crate::nicknamer::config;
    use crate::nicknamer::config::NicknamerConfig;
    use crate::nicknamer::connectors::discord::MockDiscordConnector;
    use crate::nicknamer::names::MockNamesRepository;

    // Helper function to create a test NicknamerConfig for tests
    fn create_test_config() -> config::NicknamerConfig {
        config::NicknamerConfig {
            reveal: config::RevealConfig {
                insult: "ya dingus".to_string(),
                role_to_mention: "Code Monkeys".to_string(),
            },
        }
    }

    // Common test utilities
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

    // Helper function to create a NicknamerImpl with mock objects
    fn create_nicknamer<'a>(
        repo: &'a MockNamesRepository,
        discord: &'a MockDiscordConnector,
        config: &'a NicknamerConfig,
    ) -> NicknamerImpl<'a, MockNamesRepository, MockDiscordConnector> {
        NicknamerImpl::new(repo, discord, config)
    }

    mod change_nickname_tests {
        use super::{MockRole, create_nicknamer, create_test_config};
        use crate::nicknamer::Nicknamer;
        use crate::nicknamer::connectors::discord::MockDiscordConnector;
        use crate::nicknamer::connectors::discord::server_member::ServerMemberBuilder;
        use crate::nicknamer::names::MockNamesRepository;
        use crate::nicknamer::user::Error;
        use mockall::predicate::*;

        // Guild owner ID constant for tests
        const GUILD_OWNER_ID: u64 = 987654321;

        #[tokio::test]
        async fn change_nickname_calls_discord_connector() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member
            let member = ServerMemberBuilder::new()
                .id(123456789)
                .nick_name("OldNickname")
                .user_name("TestUser")
                .is_bot(false)
                .build();

            // Set up expectations
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Ok(GUILD_OWNER_ID));

            mock_discord
                .expect_change_member_nick_name()
                .with(eq(123456789), eq("NewNickname"))
                .times(1)
                .returning(|_, _| Ok(()));

            mock_discord
                .expect_send_reply()
                .with(eq(
                    "Changed TestUser's nickname from 'OldNickname' to 'NewNickname'",
                ))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act
            let result = sut.change_nickname(&member, "NewNickname").await;

            // Assert
            assert!(result.is_ok(), "change_nickname should succeed");
        }

        #[tokio::test]
        async fn change_nickname_prevents_renaming_server_owner() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member that is the server owner
            let member = ServerMemberBuilder::new()
                .id(GUILD_OWNER_ID)
                .nick_name("OwnerNickname")
                .user_name("OwnerUsername")
                .is_bot(false)
                .build();

            // Set up expectations
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Ok(GUILD_OWNER_ID));

            // Expect admonishment message
            mock_discord
                .expect_send_reply()
                .with(eq(
                    "You dare to rename our great General Secretary??? Away with your impudence!",
                ))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act
            let result = sut.change_nickname(&member, "NewOwnerNickname").await;

            // Assert
            assert!(
                result.is_ok(),
                "change_nickname should succeed but prevent renaming"
            );
        }

        #[tokio::test]
        async fn change_nickname_handles_guild_owner_id_error() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member
            let member = ServerMemberBuilder::new()
                .id(123456789)
                .nick_name("TestNickname")
                .user_name("TestUsername")
                .is_bot(false)
                .build();

            // Set up expectations - guild owner ID returns an error
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Err(crate::nicknamer::connectors::discord::Error::CannotGetGuild));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act
            let result = sut.change_nickname(&member, "NewNickname").await;

            // Assert
            assert!(
                result.is_err(),
                "change_nickname should fail when guild owner ID cannot be retrieved"
            );
            match result {
                Err(Error::DiscordError(_)) => (),
                _ => panic!("Expected DiscordError, got different error type"),
            }
        }

        #[tokio::test]
        async fn change_nickname_handles_discord_error() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member
            let member = ServerMemberBuilder::new()
                .id(123456789)
                .nick_name("TestNickname")
                .user_name("TestUsername")
                .is_bot(false)
                .build();

            // Set up expectations
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Ok(GUILD_OWNER_ID));

            // Discord connector returns an error when changing nickname
            mock_discord
                .expect_change_member_nick_name()
                .with(eq(123456789), eq("NewNickname"))
                .times(1)
                .returning(|_, _| {
                    Err(crate::nicknamer::connectors::discord::Error::CannotGetGuild)
                });

            // Expect error message
            mock_discord
                .expect_send_reply()
                .with(eq("You fool! You messed it up!: Cannot get guild"))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act
            let result = sut.change_nickname(&member, "NewNickname").await;

            // Assert
            assert!(
                result.is_ok(),
                "change_nickname should handle Discord errors gracefully"
            );
        }

        #[tokio::test]
        async fn change_nickname_handles_permission_error() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member
            let member = ServerMemberBuilder::new()
                .id(123456789)
                .nick_name("TestNickname")
                .user_name("TestUsername")
                .is_bot(false)
                .build();

            // Set up expectations
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Ok(GUILD_OWNER_ID));

            // Discord connector returns a permission error
            mock_discord
                .expect_change_member_nick_name()
                .with(eq(123456789), eq("NewNickname"))
                .times(1)
                .returning(|_, _| {
                    Err(crate::nicknamer::connectors::discord::Error::NotEnoughPermissions)
                });

            // Expect get_role_by_name to be called
            mock_discord
                .expect_get_role_by_name()
                .with(eq(config.reveal.role_to_mention.clone()))
                .times(1)
                .returning(|_| Ok(Box::new(MockRole::new())));

            // Expect special error message for permission errors
            mock_discord
                .expect_send_reply()
                .with(eq("Some devilry restricts my power. @CodeMonkeys please investigate the rogue member <@123456789>"))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act
            let result = sut.change_nickname(&member, "NewNickname").await;

            // Assert
            assert!(
                result.is_ok(),
                "change_nickname should handle permission errors gracefully"
            );
        }

        #[tokio::test]
        async fn change_nickname_handles_empty_nickname() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member
            let member = ServerMemberBuilder::new()
                .id(123456789)
                .nick_name("TestNickname")
                .user_name("TestUsername")
                .is_bot(false)
                .build();

            // Set up expectations
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Ok(GUILD_OWNER_ID));

            // Discord connector accepts empty nickname
            mock_discord
                .expect_change_member_nick_name()
                .with(eq(123456789), eq(""))
                .times(1)
                .returning(|_, _| Ok(()));

            // Expect reply message
            mock_discord
                .expect_send_reply()
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act with empty nickname
            let result = sut.change_nickname(&member, "").await;

            // Assert
            assert!(
                result.is_ok(),
                "change_nickname should succeed with empty nickname"
            );
        }

        #[tokio::test]
        async fn change_nickname_handles_member_without_previous_nickname() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member without a nickname
            let member = ServerMemberBuilder::new()
                .id(123456789)
                .user_name("TestUsername")
                .is_bot(false)
                .build();

            // Set up expectations
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Ok(GUILD_OWNER_ID));

            // Discord connector changes nickname
            mock_discord
                .expect_change_member_nick_name()
                .with(eq(123456789), eq("FirstNickname"))
                .times(1)
                .returning(|_, _| Ok(()));

            // Expect christening message
            mock_discord
                .expect_send_reply()
                .with(eq(
                    "TestUsername has been christened with the name FirstNickname!",
                ))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act
            let result = sut.change_nickname(&member, "FirstNickname").await;

            // Assert
            assert!(
                result.is_ok(),
                "change_nickname should succeed for member without previous nickname"
            );
        }

        #[tokio::test]
        async fn change_nickname_handles_long_nickname() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member
            let member = ServerMemberBuilder::new()
                .id(123456789)
                .nick_name("OldNickname")
                .user_name("TestUsername")
                .is_bot(false)
                .build();

            // Create a very long nickname
            let long_nickname = "A".repeat(100);

            // Set up expectations
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Ok(GUILD_OWNER_ID));

            // Discord connector accepts long nickname
            mock_discord
                .expect_change_member_nick_name()
                .with(eq(123456789), eq(long_nickname.clone()))
                .times(1)
                .returning(|_, _| Ok(()));

            // Expect reply message
            mock_discord
                .expect_send_reply()
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act
            let result = sut.change_nickname(&member, &long_nickname).await;

            // Assert
            assert!(
                result.is_ok(),
                "change_nickname should succeed with long nickname"
            );
        }

        #[tokio::test]
        async fn change_nickname_handles_send_reply_error() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member
            let member = ServerMemberBuilder::new()
                .id(123456789)
                .nick_name("OldNickname")
                .user_name("TestUsername")
                .is_bot(false)
                .build();

            // Set up expectations
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Ok(GUILD_OWNER_ID));

            // Discord connector changes nickname
            mock_discord
                .expect_change_member_nick_name()
                .with(eq(123456789), eq("NewNickname"))
                .times(1)
                .returning(|_, _| Ok(()));

            // Expect send_reply to fail
            mock_discord
                .expect_send_reply()
                .times(1)
                .returning(|_| Err(crate::nicknamer::connectors::discord::Error::CannotSendReply));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act
            let result = sut.change_nickname(&member, "NewNickname").await;

            // Assert
            assert!(
                result.is_err(),
                "change_nickname should fail when send_reply fails"
            );
            match result {
                Err(Error::DiscordError(_)) => (),
                _ => panic!("Expected DiscordError, got different error type"),
            }
        }

        #[tokio::test]
        async fn change_nickname_sends_correct_message_for_nickname_change() {
            // Arrange
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Create a test member
            let member = ServerMemberBuilder::new()
                .id(123456789)
                .nick_name("OldNickname")
                .user_name("TestUsername")
                .is_bot(false)
                .build();

            // Set up expectations
            mock_discord
                .expect_get_guild_owner_id()
                .times(1)
                .returning(|| Ok(GUILD_OWNER_ID));

            // Discord connector changes nickname
            mock_discord
                .expect_change_member_nick_name()
                .with(eq(123456789), eq("NewNickname"))
                .times(1)
                .returning(|_, _| Ok(()));

            // Verify the exact message format
            mock_discord
                .expect_send_reply()
                .with(eq(
                    "Changed TestUsername's nickname from 'OldNickname' to 'NewNickname'",
                ))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Act
            let result = sut.change_nickname(&member, "NewNickname").await;

            // Assert
            assert!(
                result.is_ok(),
                "change_nickname should succeed and send correct message"
            );
        }
    }

    mod reveal_tests {
        use super::{MockRole, create_nicknamer, create_test_config};
        use crate::nicknamer::Nicknamer;
        use crate::nicknamer::connectors::discord::MockDiscordConnector;
        use crate::nicknamer::connectors::discord::server_member::ServerMemberBuilder;
        use crate::nicknamer::names::{MockNamesRepository, Names};
        use crate::nicknamer::user::Error;
        use mockall::predicate::*;
        use std::collections::HashMap;

        #[tokio::test]
        async fn can_successfully_reveal_all_members() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

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

            // Expect exact message content
            mock_discord
                .expect_send_reply()
                .with(eq(format!(
                    "Here are people's real names, {}:
\t'AliceNickname' is Alice
\t'BobNickname' is Bob",
                    config.reveal.insult
                )))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal_all().await;

            // Verify results
            assert!(result.is_ok(), "reveal_all should succeed");
        }

        #[tokio::test]
        async fn handles_discord_error_when_revealing_all() {
            // Setup mock objects
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Setup discord connector to return an error
            mock_discord
                .expect_get_members_of_current_channel()
                .times(1)
                .returning(|| {
                    Err(crate::nicknamer::connectors::discord::Error::NotInServerChannel)
                });

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal_all().await;

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
            let config = create_test_config();

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

            // Expect exact message content
            mock_discord
                .expect_send_reply()
                .with(eq(
                    "Hey @CodeMonkeys, these members are unrecognized:
                \tHumanUser aka 'HumanUser'
                One of y'all should improve real name management and/or add them to the config"
                        .to_string(),
                ))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal_all().await;

            // Verify results
            assert!(result.is_ok(), "reveal_all should succeed");
        }

        #[tokio::test]
        async fn reveal_all_with_all_bot_users_should_not_send_message() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

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
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal_all().await;

            // Verify results
            assert!(result.is_ok(), "reveal_all should succeed");
        }

        #[tokio::test]
        async fn reveal_member_should_handle_bot_with_nickname() {
            // Setup mock objects
            let mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

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
                .with(eq(format!(
                    "BotNick is a bot, {}!",
                    create_test_config().reveal.insult
                )))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal(&bot_member).await;

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
            let config = create_test_config();

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
                    create_test_config().reveal.insult
                )))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal(&bot_member).await;

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
            let config = create_test_config();

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
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal(&bot_member).await;

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
            let config = create_test_config();

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
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal(&member).await;

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
            let config = create_test_config();

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
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal(&member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal should succeed for a member with real name"
            );
        }

        #[tokio::test]
        async fn reveal_member_should_handle_member_with_real_name_but_no_nickname() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Define test data - a user with a real name but no nickname
            let member = ServerMemberBuilder::new()
                .id(111111111)
                .user_name("UserWithoutNickname")
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

            // The message should include the real name but use username instead of nickname
            mock_discord
                .expect_send_reply()
                .with(eq("'UserWithoutNickname' is Real Person"))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal(&member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal should succeed for a member with real name but no nickname"
            );
        }

        #[tokio::test]
        async fn reveal_member_should_handle_member_with_neither_real_name_nor_nickname() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mut mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

            // Define test data - a user with neither real name nor nickname
            let member = ServerMemberBuilder::new()
                .id(111111111)
                .user_name("UserWithoutAnything")
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

            // The message should indicate the user has neither a nickname nor a real name
            mock_discord
                .expect_send_reply()
                .with(eq(
                    "UserWithoutAnything has neither a nickname nor a real name",
                ))
                .times(1)
                .returning(|_| Ok(()));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal(&member).await;

            // Verify results
            assert!(
                result.is_ok(),
                "reveal should succeed for a member with neither real name nor nickname"
            );
        }

        #[tokio::test]
        async fn reveal_member_should_handle_names_repository_error() {
            // Setup mock objects
            let mut mock_repo = MockNamesRepository::new();
            let mock_discord = MockDiscordConnector::new();
            let config = create_test_config();

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
                .returning(|| Err(crate::nicknamer::names::Error::CannotLoadNames));

            // Create nicknamer with mock objects
            let sut = create_nicknamer(&mock_repo, &mock_discord, &config);

            // Execute the method under test
            let result = sut.reveal(&member).await;

            // Verify results
            assert!(result.is_err(), "reveal should fail when repository fails");
            assert!(
                matches!(result.unwrap_err(), Error::NamesAccessError(_)),
                "Error should be a NamesAccessError"
            );
        }
    }
}
