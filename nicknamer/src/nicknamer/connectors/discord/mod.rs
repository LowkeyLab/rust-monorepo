//! Discord connectivity module for the nickname manager.
//!
//! This module provides abstractions for interacting with Discord, including:
//! - Error types for Discord connectivity issues
//! - Traits defining Discord server interaction capabilities
//! - Data structures for representing Discord server members and roles
//!
//! The module is designed to be implementation-agnostic, allowing for different
//! Discord client libraries to be used by implementing the `DiscordConnector` trait.
//! A concrete implementation using the Serenity library is provided in the `serenity` submodule.

use mockall::automock;
use thiserror::Error;

pub(crate) mod serenity;

/// Errors that can occur during Discord connectivity operations.
///
/// These errors represent various failure modes when interacting with Discord,
/// such as being unable to find channels, members, or send messages.
#[derive(Error, Debug)]
pub enum Error {
    /// The command was not executed in a server channel
    #[error("Not in a server channel")]
    NotInServerChannel,
    /// Unable to find the specified Discord channel
    #[error("Cannot find channel")]
    CannotFindChannel,
    /// Unable to retrieve the members of a channel
    #[error("Cannot find members of channel")]
    CannotFindMembersOfChannel,
    /// Failed to send a reply message
    #[error("Cannot send reply")]
    CannotSendReply,
    /// Failed to retrieve the guild (server) information
    #[error("Cannot get guild")]
    CannotGetGuild,
    /// Unable to find the specified role
    #[error("Cannot find role")]
    CannotFindRole,
    #[error("Not enough permissions")]
    NotEnoughPermissions,
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
    /// Looks up a role in the current guild by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the role to find
    ///
    /// # Returns
    ///
    /// * `Result<Box<dyn Role>, Error>` - The role if found, or an error otherwise
    async fn get_role_by_name(&self, name: &str) -> Result<Box<dyn Role>, Error>;

    async fn change_member_nick_name<'connector, 'name>(
        &'connector self,
        member_id: u64,
        new_nick_name: &'name str,
    ) -> Result<(), Error>;

    async fn get_guild_owner_id(&self) -> Result<u64, Error>;
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
    /// Whether the member is a bot
    pub(crate) is_bot: bool,
    pub(crate) mention: String,
}

/// Builder for ServerMember instances.
///
/// This provides a fluent interface for constructing ServerMember objects,
/// making test code more readable and flexible.
#[derive(Debug, Default)]
pub struct ServerMemberBuilder {
    id: u64,
    nick_name: Option<String>,
    user_name: String,
    is_bot: bool,
    mention: String,
}

impl ServerMemberBuilder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Discord user ID.
    pub fn id(mut self, id: u64) -> Self {
        self.id = id;
        // Default the mention to a standard Discord mention format if not explicitly set
        if self.mention.is_empty() {
            self.mention = format!("<@{}>", id);
        }
        self
    }

    /// Sets the nickname for this server member.
    pub fn nick_name(mut self, nick_name: impl Into<String>) -> Self {
        self.nick_name = Some(nick_name.into());
        self
    }

    /// Sets the username for this server member.
    pub fn user_name(mut self, user_name: impl Into<String>) -> Self {
        self.user_name = user_name.into();
        self
    }

    /// Sets whether this server member is a bot.
    pub fn is_bot(mut self, is_bot: bool) -> Self {
        self.is_bot = is_bot;
        self
    }

    /// Sets the mention string for this server member.
    pub fn mention(mut self, mention: impl Into<String>) -> Self {
        self.mention = mention.into();
        self
    }

    /// Builds a ServerMember instance with the configured values.
    pub fn build(self) -> ServerMember {
        ServerMember {
            id: self.id,
            nick_name: self.nick_name,
            user_name: self.user_name,
            is_bot: self.is_bot,
            mention: self.mention,
        }
    }
}

/// Represents an entity that can be mentioned in Discord messages.
///
/// This trait is implemented by types that can be referenced in Discord
/// messages using mentions (like @user or @role).
pub trait Mentionable: Send + Sync + 'static {
    /// Returns the string representation of a mention for this entity.
    ///
    /// # Returns
    ///
    /// The formatted mention string that can be included in Discord messages
    fn mention(&self) -> String;
}

/// Represents a Discord role.
///
/// This trait extends the `Mentionable` trait to specifically identify
/// Discord role entities. Implementations should represent Discord roles
/// with their associated permissions and properties.
pub trait Role: Mentionable {}
