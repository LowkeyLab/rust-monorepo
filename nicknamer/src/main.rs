mod nicknamer;

use self::nicknamer::commands::reveal;
use crate::nicknamer::commands;
use crate::nicknamer::discord;
use crate::nicknamer::discord::DiscordConnector;
use crate::nicknamer::file;
use log::{LevelFilter, info};
use log4rs::Config;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use poise::serenity_prelude as serenity;
use std::collections::HashMap;

/// Ping command to test bot availability
///
/// Any instance of bot connected to the server will respond with "Pong!" and some runtime information.
#[poise::command(prefix_command)]
async fn ping(ctx: discord::serenity::Context<'_>) -> Result<(), discord::serenity::Error> {
    ctx.reply("Pong!").await?;
    Ok(())
}

/// Show this menu
#[poise::command(prefix_command)]
pub async fn help(
    ctx: discord::serenity::Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), discord::serenity::Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type ~help command for more info on a command.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Routine responsible for 'nick' discord command.
#[poise::command(prefix_command)]
async fn nick(
    _ctx: discord::serenity::Context<'_>,
    member: serenity::Member,
) -> Result<(), discord::serenity::Error> {
    commands::nick(member.user.id);
    Ok(())
}

/// Reveal members' true names, greatly diminishing their power level
///
/// Specifically, I'll reveal the names of members that can access this channel \
/// You can also tag another member and I'll only reveal the real name of that person
#[poise::command(prefix_command)]
async fn reveal(
    ctx: discord::serenity::Context<'_>,
    #[description = "The specific member to reveal the name of"] member: Option<serenity::Member>,
) -> Result<(), discord::serenity::Error> {
    info!("Revealing nicknames for current channel members ...");
    let real_names = file::RealNames::from_embedded_yaml()?;
    info!("Loaded {} real names", real_names.names.len());
    let connector = discord::serenity::SerenityDiscordConnector::new(ctx);
    let members = connector.get_members_of_current_channel().await?;
    info!("Found {} members in current channel", members.len());
    let users: HashMap<u64, commands::User> = members
        .iter()
        .filter_map(|member| {
            let Some(real_name) = real_names.names.get(&member.id) else {
                return None;
            };
            let user = commands::User::from_discord_server_member(member, real_name.clone());
            Some((user.id, user))
        })
        .collect();
    info!("Found {} users with real names", users.len());
    let real_names = commands::RealNames { users };
    match member {
        Some(member) => {
            ctx.reply(reveal::reveal_member(member.user.id.get(), &real_names)?)
                .await?
        }
        None => ctx.reply(reveal::reveal(&real_names)?).await?,
    };
    Ok(())
}

#[tokio::main]
async fn main() {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("nicknamer", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    let _log4rs_handle = log4rs::init_config(config).unwrap();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let framework =
        poise::Framework::<discord::serenity::Data, discord::serenity::Error>::builder()
            .options(poise::FrameworkOptions {
                commands: vec![help(), ping(), reveal()],
                prefix_options: poise::PrefixFrameworkOptions {
                    prefix: Some("~".into()),
                    ..Default::default()
                },
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(discord::serenity::Data {})
                })
            })
            .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    info!("Starting bot...");
    client.unwrap().start().await.unwrap();
}
