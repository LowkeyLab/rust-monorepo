mod nicknamer;

use self::nicknamer::commands::names::EmbeddedNamesRepository;
use self::nicknamer::commands::names::Names;
use self::nicknamer::commands::reveal;
use self::nicknamer::connectors::discord;
use self::nicknamer::connectors::discord::DiscordConnector;
use self::nicknamer::connectors::discord::serenity::{Context, SerenityDiscordConnector};
use self::nicknamer::connectors::file::RealNames;
use crate::nicknamer::commands;
use crate::nicknamer::commands::reveal::{Revealer, RevealerImpl};
use crate::nicknamer::connectors::discord::ServerMember;
use log::{LevelFilter, info};
use log4rs::Config;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Member;

/// Ping command to test bot availability
///
/// Any instance of bot connected to the server will respond with "Pong!" and some runtime information.
#[poise::command(prefix_command)]
async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
    ctx.reply("Pong!").await?;
    Ok(())
}

/// Show this menu
#[poise::command(prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> anyhow::Result<()> {
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
) -> anyhow::Result<()> {
    commands::nick(member.user.id);
    Ok(())
}

/// Reveal members' true names, greatly diminishing their power level
///
/// Specifically, I'll reveal the names of members that can access this channel
///
/// You can also tag another member and I'll only reveal the real name of that person
#[poise::command(prefix_command)]
async fn reveal(
    ctx: Context<'_>,
    #[description = "The specific member to reveal the name of"] member: Option<Member>,
) -> anyhow::Result<()> {
    let name_repository = EmbeddedNamesRepository::new();
    let connector = SerenityDiscordConnector::new(ctx);
    let revealer = RevealerImpl::new(&name_repository, &connector);
    match member {
        Some(member) => reveal_single_member(&revealer, &member.into()).await,
        None => reveal_all_members(&revealer).await,
    }
}

async fn reveal_all_members<T: Revealer>(revealer: &T) -> anyhow::Result<()> {
    let _ = revealer.reveal_all().await?;
    Ok(())
}

async fn reveal_single_member<T: Revealer>(
    revealer: &T,
    _member: &ServerMember,
) -> anyhow::Result<()> {
    let _ = revealer.reveal_all().await?;
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

    let framework = poise::Framework::<discord::serenity::Data, anyhow::Error>::builder()
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
