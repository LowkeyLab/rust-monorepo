use self::serenity::Error;

pub(crate) mod serenity;

pub trait DiscordConnector {
    async fn get_members_of_current_channel(&self) -> Result<Vec<ServerMember>, Error>;
}

pub struct ServerMember {
    pub(crate) id: u64,
    pub(crate) nick_name: Option<String>,
    pub(crate) user_name: String,
}
