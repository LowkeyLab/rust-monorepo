use crate::nicknamer::commands::Error;
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
        self.discord_connector
            .change_member_nick_name(member.id, new_nick_name)
            .await?;
        match &member.nick_name {
            Some(nick_name) => {
                let reply = format!(
                    "Changed {}'s nickname from '{}' to '{}'",
                    member.user_name, nick_name, new_nick_name
                );
                self.discord_connector.send_reply(&reply).await?;
            }
            None => {
                let reply = format!(
                    "{} has been christened with the name {}!",
                    member.user_name, new_nick_name
                );
                self.discord_connector.send_reply(&reply).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nicknamer::connectors::discord::Error as DiscordError;
    use crate::nicknamer::connectors::discord::MockDiscordConnector;
    use mockall::predicate::*;

    #[tokio::test]
    async fn nick_service_calls_discord_connector() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
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
    async fn nick_service_handles_discord_error() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
        mock_discord
            .expect_change_member_nick_name()
            .times(1)
            .returning(|_, _| Err(DiscordError::NotEnoughPermissions));

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
            Err(Error::DiscordError(_)) => (),
            _ => panic!("Expected DiscordError, got different error type"),
        }
    }

    #[tokio::test]
    async fn nick_service_handles_empty_nickname() {
        // Arrange
        let mut mock_discord = MockDiscordConnector::new();
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
}
