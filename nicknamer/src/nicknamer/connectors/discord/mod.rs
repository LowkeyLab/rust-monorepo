use mockall::automock;
use thiserror::Error;

pub(crate) mod serenity;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not in a server channel")]
    NotInServerChannel,
    #[error("Cannot find channel")]
    CannotFindChannel,
    #[error("Cannot find members of channel")]
    CannotFindMembersOfChannel,
    #[error("Cannot send reply")]
    CannotSendReply,
    #[error("Cannot get guild")]
    CannotGetGuild,
    #[error("Cannot find role")]
    CannotFindRole,
}

/// Trait for abstracting Discord server interactions.
///
/// This trait defines the required functionality for connecting to
/// and retrieving information from Discord servers.
#[automock]
pub trait DiscordConnector {
    /// Retrieves all members present in the current Discord channel.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<ServerMember>, Error>` - List of server members on success, or Discord error
    async fn get_members_of_current_channel(&self) -> Result<Vec<ServerMember>, Error>;
    /// Sends a reply to the person that invoked the prefix command
    async fn send_reply(&self, message: &str) -> Result<(), Error>;
    async fn get_role_by_name(&self, name: &str) -> Result<Box<dyn Role>, Error>;
}

/// Represents a member of a Discord server.
///
/// Contains basic information about a Discord server member,
/// including their ID, nickname (if any), and username.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ServerMember {
    /// Discord user's unique identifier
    pub(crate) id: u64,
    /// Optional nickname set for the user in the server
    pub(crate) nick_name: Option<String>,
    /// Discord username of the member
    pub(crate) user_name: String,
}

pub trait Mentionable: Send + Sync + 'static {
    fn mention(&self) -> String;
}

pub trait Role: Mentionable {}
