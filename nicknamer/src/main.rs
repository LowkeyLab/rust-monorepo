mod nicknamer;

use self::nicknamer::discord::serenity::{Context, Data, Error};
use crate::nicknamer::commands;
use crate::nicknamer::discord;
use crate::nicknamer::discord::DiscordConnector;
use crate::nicknamer::file;
use log::{LevelFilter, info};
use log4rs::Config;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use poise::serenity_prelude as serenity;

/// Ping command to test bot availability
///
/// Any instance of bot connected to the server will respond with "Pong!" and some runtime information.
#[poise::command(prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Pong!").await?;
    Ok(())
}

/// Show this menu
#[poise::command(prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
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
async fn nick(_ctx: Context<'_>, member: serenity::Member) -> Result<(), Error> {
    commands::nick(member.user.id);
    Ok(())
}

#[poise::command(prefix_command)]
async fn reveal(ctx: Context<'_>) -> Result<(), Error> {
    let real_names = file::RealNames::from_embedded_yaml()?;
    let connector = discord::serenity::SerenityDiscordConnector::new(ctx);
    let members = connector.get_members_of_current_channel().await?;
    let users = members
        .iter()
        .filter(|member| real_names.names.contains_key(&member.id))
        .map(|member| commands::User {
            id: member.id,
            display_name: member
                .nick_name
                .clone()
                .unwrap_or_else(|| member.user_name.clone()),
            real_name: real_names.names.get(&member.id).unwrap().clone(),
        })
        .collect::<Vec<_>>();
    let real_names = commands::RealNames { users };
    ctx.reply(commands::reveal(&real_names)?).await?;
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
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::<Data, Error>::builder()
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
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    info!("Starting bot...");
    client.unwrap().start().await.unwrap();
}
