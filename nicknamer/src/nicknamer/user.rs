use crate::nicknamer::connectors::discord;
use crate::nicknamer::connectors::discord::server_member;
use crate::nicknamer::names;
use thiserror::Error;

#[derive(Debug, PartialEq, Default)]
pub struct User {
    pub id: u64,
    pub user_name: String,
    pub nick_name: Option<String>,
    pub real_name: Option<String>,
}
impl From<server_member::ServerMember> for User {
    fn from(discord_member: server_member::ServerMember) -> Self {
        Self {
            id: discord_member.id,
            user_name: discord_member.user_name.clone(),
            nick_name: discord_member.nick_name.clone(),
            real_name: None,
        }
    }
}

impl From<&server_member::ServerMember> for User {
    fn from(discord_member: &server_member::ServerMember) -> Self {
        Self {
            id: discord_member.id,
            user_name: discord_member.user_name.clone(),
            nick_name: discord_member.nick_name.clone(),
            real_name: None,
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Something went wrong with Discord")]
    DiscordError(#[from] discord::Error),
    #[error("Something went wrong getting people's names")]
    NamesAccessError(#[from] names::Error),
}
