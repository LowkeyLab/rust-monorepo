pub(crate) mod serenity;

pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub trait DiscordConnector {
    async fn get_members_of_current_channel(&self) -> Result<Vec<ServerMember>, Error>;
}

pub struct ServerMember {
    pub(crate) id: u64,
    pub(crate) nick_name: Option<String>,
    pub(crate) user_name: String,
}
