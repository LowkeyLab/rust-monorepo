use crate::nicknamer::discord::{Context, DiscordConnector, Error, ServerMember};

pub struct SerenityDiscordConnector<'a> {
    context: Context<'a>,
}

impl<'a> SerenityDiscordConnector<'a> {
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
