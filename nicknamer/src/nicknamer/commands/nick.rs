use crate::nicknamer::commands::Error;
use crate::nicknamer::connectors::discord::DiscordConnector;

pub trait NickService {
    async fn nick(&self, user_id: u64, new_nick_name: &str) -> Result<(), Error>;
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
    async fn nick(&self, user_id: u64, new_nick_name: &str) -> Result<(), Error> {
        self.discord_connector
            .change_member_nick_name(user_id, new_nick_name)
            .await?;
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

        let service = NickServiceImpl::new(&mock_discord);

        // Act
        let result = service.nick(123456789, "NewNick").await;

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

        // Act
        let result = service.nick(123456789, "NewNick").await;

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

        let service = NickServiceImpl::new(&mock_discord);

        // Act
        let result = service.nick(123456789, "").await;

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

        let service = NickServiceImpl::new(&mock_discord);

        // Act
        let result = service.nick(123456789, &long_nickname).await;

        // Assert
        assert!(result.is_ok());
    }
}
