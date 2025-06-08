use crate::nicknamer::commands::Error;
use crate::nicknamer::config;
use crate::nicknamer::connectors::discord;
use crate::nicknamer::connectors::discord::DiscordConnector;
use crate::nicknamer::connectors::discord::server_member::ServerMember;

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
