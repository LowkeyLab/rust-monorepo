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
#[cfg(test)]
pub struct ServerMemberBuilder {
    id: u64,
    nick_name: Option<String>,
    user_name: String,
    is_bot: bool,
    mention: String,
}

#[cfg(test)]
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
    #[allow(clippy::wrong_self_convention)]
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
