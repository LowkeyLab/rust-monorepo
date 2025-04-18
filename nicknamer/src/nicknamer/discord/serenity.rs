//! Serenity-based implementation of Discord connectivity.
//!
//! This module provides the concrete implementation of the Discord connector
//! trait using the Serenity Discord library.

use crate::nicknamer::discord::{DiscordConnector, ServerMember};

/// Discord connector implementation using Serenity library.
///
/// Provides functionality to interact with Discord servers using
/// the Serenity context.
pub struct SerenityDiscordConnector<'a> {
    context: Context<'a>,
}

impl<'a> SerenityDiscordConnector<'a> {
    /// Creates a new SerenityDiscordConnector instance.
    ///
    /// # Arguments
    ///
    /// * `context` - Poise command context for Discord interactions
    pub fn new(context: Context<'a>) -> Self {
        Self { context }
    }
}

impl DiscordConnector for SerenityDiscordConnector<'_> {
    async fn get_members_of_current_channel(&self) -> Result<Vec<ServerMember>, Error> {
        let ctx = &self.context;
        let channel = ctx.channel_id().to_channel(ctx).await?;
        let Some(channel) = channel.guild() else {
            return Err("You're not in a discord server's channel".into());
        };
        let members = channel.members(ctx)?;
        let members = members
            .iter()
            .map(|member| ServerMember {
                id: member.user.id.get(),
                nick_name: member.nick.clone(),
                user_name: member.user.name.clone(),
            })
            .collect();
        Ok(members)
    }
}

/// Empty data structure for Poise framework configuration
pub struct Data {}

/// Type alias for error handling in Discord operations
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
/// Type alias for Poise command context
pub type Context<'a> = poise::Context<'a, Data, Error>;
