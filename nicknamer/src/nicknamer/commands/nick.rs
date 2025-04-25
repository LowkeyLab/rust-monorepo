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
