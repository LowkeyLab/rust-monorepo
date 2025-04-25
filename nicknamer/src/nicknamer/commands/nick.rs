use crate::nicknamer::commands::Error;
use crate::nicknamer::config;
use crate::nicknamer::connectors::discord;
use crate::nicknamer::connectors::discord::{DiscordConnector, ServerMember};

pub trait NickService {
    async fn nick(&self, member: &ServerMember, new_nick_name: &str) -> Result<(), Error>;
}

pub struct NickServiceImpl<'a, DISCORD: DiscordConnector> {
    discord_connector: &'a DISCORD,
}

impl<'a, DISCORD: DiscordConnector> NickServiceImpl<'a, DISCORD> {
    pub fn new(discord_connector: &'a DISCORD) -> Self {
        Self { discord_connector }
    }
}

impl<'a, DISCORD: DiscordConnector> NickService for NickServiceImpl<'a, DISCORD> {
    async fn nick(&self, member: &ServerMember, new_nick_name: &str) -> Result<(), Error> {
        let owner_id = self.discord_connector.get_guild_owner_id().await?;
        if member.id == owner_id {
            self.admonish_for_violating_party_guidelines().await?;
        } else {
            self.change_member_nick_name(&member, new_nick_name).await?;
        }
        Ok(())
    }
}

impl<'a, DISCORD: DiscordConnector> NickServiceImpl<'a, DISCORD> {
    async fn admonish_for_violating_party_guidelines(&self) -> Result<(), Error> {
        let reply = "You dare to rename our great General Secretary??? Away with your impudence!";
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

impl<'a, DISCORD: DiscordConnector> NickServiceImpl<'a, DISCORD> {
    async fn change_member_nick_name(
        &self,
        member: &ServerMember,
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
                            .get_role_by_name(config::CODE_MONKEYS_ROLE_NAME)
                            .await?;
                        format!(
                            "Some devilry restricts my power. {} please investigate the rogue member",
                            role_to_mention.mention()
                        )
                    }
                    err => format!("You fool! You messed it up!: {}", err),
                };
                self.discord_connector.send_reply(&reply).await?;
            }
        }
        Ok(())
    }
}

impl<'a, DISCORD: DiscordConnector> NickServiceImpl<'a, DISCORD> {
    async fn send_reply_for_member_without_nick_name(
        &self,
        member: &ServerMember,
        new_nick_name: &str,
    ) -> Result<(), Error> {
        let reply = format!(
            "{} has been christened with the name {}!",
            member.user_name, new_nick_name
        );
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

impl<'a, DISCORD: DiscordConnector> NickServiceImpl<'a, DISCORD> {
    async fn send_reply_for_member_with_nick_name(
        &self,
        member: &ServerMember,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nicknamer::connectors::discord::MockDiscordConnector;
    use crate::nicknamer::connectors::discord::{Error as DiscordError, Mentionable, Role};
    use mockall::predicate::*;

    // Guild owner ID constant for tests
    const GUILD_OWNER_ID: u64 = 987654321;

    #[tokio::test]
    async fn nick_service_calls_discord_connector() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        mock_discord
            .expect_change_member_nick_name()
            .with(eq(123456789), eq("NewNick"))
            .times(1)
            .returning(|_, _| Ok(()));

        mock_discord
            .expect_send_reply()
            .times(1)
            .returning(|_| Ok(()));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn nick_service_prevents_renaming_server_owner() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();

        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID));

        // Expect admonish message to be sent
        mock_discord
            .expect_send_reply()
            .with(eq(
                "You dare to rename our great General Secretary??? Away with your impudence!",
            ))
            .times(1)
            .returning(|_| Ok(()));

        // Should NOT call change_member_nick_name for server owner
        mock_discord.expect_change_member_nick_name().times(0);

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: GUILD_OWNER_ID, // Same as owner_id
            nick_name: Some("OwnerNick".to_string()),
            user_name: "OwnerName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn nick_service_handles_guild_owner_id_error() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Err(DiscordError::CannotGetGuild));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(Error::DiscordError(DiscordError::CannotGetGuild)) => (),
            _ => panic!("Expected CannotGetGuild error, got different error type"),
        }
    }

    #[tokio::test]
    async fn nick_service_handles_admonish_error() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();

        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID));

        // Simulate error when sending admonishment
        mock_discord
            .expect_send_reply()
            .times(1)
            .returning(|_| Err(DiscordError::CannotSendReply));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: GUILD_OWNER_ID,
            nick_name: Some("OwnerNick".to_string()),
            user_name: "OwnerName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(Error::DiscordError(DiscordError::CannotSendReply)) => (),
            _ => panic!("Expected CannotSendReply error, got different error type"),
        }
    }

    // Mock implementation of Role for testing
    struct TestRole;

    impl Mentionable for TestRole {
        fn mention(&self) -> String {
            "@Code Monkeys".to_string()
        }
    }

    impl Role for TestRole {}

    #[tokio::test]
    async fn nick_service_handles_discord_error() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        mock_discord
            .expect_change_member_nick_name()
            .times(1)
            .returning(|_, _| Err(DiscordError::NotEnoughPermissions));

        // Add the missing expectation for get_role_by_name
        mock_discord
            .expect_get_role_by_name()
            .with(eq(config::CODE_MONKEYS_ROLE_NAME))
            .times(1)
            .returning(|_| Ok(Box::new(TestRole)));

        // Make send_reply fail to propagate the error
        mock_discord
            .expect_send_reply()
            .times(1)
            .returning(|_| Err(DiscordError::CannotSendReply));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(Error::DiscordError(DiscordError::CannotSendReply)) => (),
            _ => panic!("Expected DiscordError::CannotSendReply error, got different error type"),
        }
    }

    #[tokio::test]
    async fn nick_service_handles_permission_error_successfully() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        mock_discord
            .expect_change_member_nick_name()
            .times(1)
            .returning(|_, _| Err(DiscordError::NotEnoughPermissions));

        // Expect get_role_by_name to be called with the Code Monkeys role name
        mock_discord
            .expect_get_role_by_name()
            .with(eq(config::CODE_MONKEYS_ROLE_NAME))
            .times(1)
            .returning(|_| Ok(Box::new(TestRole)));

        // Expect send_reply to succeed with the proper message
        mock_discord
            .expect_send_reply()
            .with(eq("Some devilry restricts my power. @Code Monkeys please investigate the rogue member"))
            .times(1)
            .returning(|_| Ok(()));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn nick_service_handles_error_when_fetching_role() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        mock_discord
            .expect_change_member_nick_name()
            .times(1)
            .returning(|_, _| Err(DiscordError::NotEnoughPermissions));

        // Simulate failure to get the Code Monkeys role
        mock_discord
            .expect_get_role_by_name()
            .with(eq(config::CODE_MONKEYS_ROLE_NAME))
            .times(1)
            .returning(|_| Err(DiscordError::CannotFindRole));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(Error::DiscordError(DiscordError::CannotFindRole)) => (),
            _ => panic!("Expected DiscordError::CannotFindRole error, got different error type"),
        }
    }

    #[tokio::test]
    async fn nick_service_handles_empty_nickname() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        mock_discord
            .expect_change_member_nick_name()
            .with(eq(123456789), eq(""))
            .times(1)
            .returning(|_, _| Ok(()));

        mock_discord
            .expect_send_reply()
            .times(1)
            .returning(|_| Ok(()));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "").await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn nick_service_handles_long_nickname() {
        // Arrange
        let long_nickname = "A".repeat(100); // Some very long nickname
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        mock_discord
            .expect_change_member_nick_name()
            .with(eq(123456789), eq(String::from(long_nickname.as_str())))
            .times(1)
            .returning(|_, _| Ok(()));

        mock_discord
            .expect_send_reply()
            .times(1)
            .returning(|_| Ok(()));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, &long_nickname).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn nick_service_handles_member_without_previous_nickname() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        mock_discord
            .expect_change_member_nick_name()
            .with(eq(123456789), eq("FirstNick"))
            .times(1)
            .returning(|_, _| Ok(()));

        // Verify the correct christening message is sent
        mock_discord
            .expect_send_reply()
            .with(eq("UserName has been christened with the name FirstNick!"))
            .times(1)
            .returning(|_| Ok(()));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: None, // No previous nickname
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "FirstNick").await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn nick_service_handles_send_reply_error() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        mock_discord
            .expect_change_member_nick_name()
            .times(1)
            .returning(|_, _| Ok(()));

        // Simulate error when sending reply
        mock_discord
            .expect_send_reply()
            .times(1)
            .returning(|_| Err(DiscordError::CannotSendReply));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(Error::DiscordError(DiscordError::CannotSendReply)) => (),
            _ => panic!("Expected CannotSendReply error, got different error type"),
        }
    }

    #[tokio::test]
    async fn nick_service_sends_correct_message_for_nickname_change() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        mock_discord
            .expect_change_member_nick_name()
            .times(1)
            .returning(|_, _| Ok(()));

        // Verify the exact message format for existing nickname
        mock_discord
            .expect_send_reply()
            .with(eq(
                "Changed UserName's nickname from 'OldNick' to 'NewNick'",
            ))
            .times(1)
            .returning(|_| Ok(()));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn nick_service_handles_other_discord_errors() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_get_guild_owner_id()
            .times(1)
            .returning(|| Ok(GUILD_OWNER_ID)); // Different from member ID

        // Return a different Discord error than NotEnoughPermissions
        mock_discord
            .expect_change_member_nick_name()
            .times(1)
            .returning(|_, _| Err(DiscordError::CannotGetGuild));

        // Check that the generic error message is used
        mock_discord
            .expect_send_reply()
            .with(eq("You fool! You messed it up!: Cannot get guild"))
            .times(1)
            .returning(|_| Ok(()));

        let service = NickServiceImpl::new(&mock_discord);

        let member = ServerMember {
            id: 123456789,
            nick_name: Some("OldNick".to_string()),
            user_name: "UserName".to_string(),
            is_bot: false,
        };

        // Act
        let result = service.nick(&member, "NewNick").await;

        // Assert
        assert!(result.is_ok());
    }
}
