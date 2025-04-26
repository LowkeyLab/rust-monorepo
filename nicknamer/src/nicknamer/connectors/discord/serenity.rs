//! Serenity-based implementation of Discord connectivity.
//!
//! This module provides the concrete implementation of the Discord connector
//! trait using the Serenity Discord library.

use crate::nicknamer::connectors::discord::Error::{
    CannotFindChannel, CannotFindMembersOfChannel, CannotFindRole, CannotGetGuild, CannotSendReply,
    NotEnoughPermissions, NotInServerChannel,
};
use crate::nicknamer::connectors::discord::server_member::ServerMember;
use crate::nicknamer::connectors::discord::{DiscordConnector, Error, Mentionable, Role};
use log::info;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::EditMember;
use poise::serenity_prelude::Mentionable as poise_Mentionable;

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
        let Ok(channel) = ctx.channel_id().to_channel(ctx).await else {
            return Err(CannotFindChannel);
        };
        let Some(channel) = channel.guild() else {
            return Err(NotInServerChannel);
        };
        let Ok(members) = channel.members(ctx) else {
            return Err(CannotFindMembersOfChannel);
        };
        let members: Vec<ServerMember> =
            members.iter().map(|member| member.clone().into()).collect();
        info!("Found {} members in current channel", members.len());
        Ok(members)
    }

    async fn send_reply(&self, message: &str) -> Result<(), Error> {
        let ctx = &self.context;
        let Ok(_) = ctx.reply(message).await else {
            return Err(CannotSendReply);
        };
        Ok(())
    }

    async fn get_role_by_name(&self, name: &str) -> Result<Box<dyn Role>, Error> {
        let Some(guild) = self.context.guild() else {
            return Err(CannotGetGuild);
        };
        let Some(role) = guild.role_by_name(name) else {
            return Err(CannotFindRole);
        };
        Ok(Box::new(role.clone()))
    }

    async fn change_member_nick_name(
        &self,
        member_id: u64,
        new_nick_name: &str,
    ) -> Result<(), Error> {
        let Some(guild) = self.context.guild_id() else {
            return Err(CannotGetGuild);
        };
        let builder = EditMember::new().nickname(new_nick_name);
        let Ok(_member) = guild.edit_member(self.context, member_id, builder).await else {
            return Err(NotEnoughPermissions);
        };
        Ok(())
    }

    async fn get_guild_owner_id(&self) -> Result<u64, Error> {
        let Some(guild) = self.context.guild() else {
            return Err(CannotGetGuild);
        };
        Ok(guild.owner_id.get())
    }
}

impl Mentionable for serenity::Role {
    fn mention(&self) -> String {
        <Self as serenity::Mentionable>::mention(&self).to_string()
    }
}

impl Role for serenity::Role {}

impl From<serenity::Member> for ServerMember {
    fn from(member: serenity::Member) -> Self {
        ServerMember {
            id: member.user.id.get(),
            nick_name: member.nick.clone(),
            user_name: member.user.name.clone(),
            is_bot: member.user.bot,
            mention: member.mention().to_string(),
        }
    }
}

/// Empty data structure for Poise framework configuration
pub struct Data {}

/// Type alias for Poise command context
pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;
