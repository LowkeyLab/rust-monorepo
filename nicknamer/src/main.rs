mod nicknamer;

use self::nicknamer::config::Config;
use self::nicknamer::connectors::discord;
use self::nicknamer::connectors::discord::serenity::{Context, SerenityDiscordConnector};
use self::nicknamer::names::EmbeddedNamesRepository;
use crate::nicknamer::{Nicknamer, NicknamerImpl};
use log::{LevelFilter, debug, info};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{FullEvent, Member, Message};

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

/// Changes the nickname for a member into a new one
#[poise::command(prefix_command)]
async fn nick(
    ctx: Context<'_>,
    #[description = "The specific member to reveal the name of"] member: Member,
    #[description = "The new nickname to set"] nickname: String,
) -> anyhow::Result<()> {
    let connector = SerenityDiscordConnector::new(ctx);
    let nicknamer = NicknamerImpl::new(
        &ctx.data().names_repository,
        &connector,
        ctx.data().config.nicknamer.clone(),
    );
    nicknamer.change_nickname(&member.into(), &nickname).await?;
    Ok(())
}

/// Reveal members' true names, greatly diminishing their power level
///
/// Specifically, I'll reveal the names of members that can access this channel
///
/// You can also tag another member and I'll reveal the name of that person, regardless of whether they can access this channel or not
#[poise::command(prefix_command)]
async fn reveal(
    ctx: Context<'_>,
    #[description = "The specific member to reveal the name of"] member: Option<Member>,
) -> anyhow::Result<()> {
    // Use the names_repository from the Data struct via the wrapper
    let connector = SerenityDiscordConnector::new(ctx);
    let nicknamer = NicknamerImpl::new(
        &ctx.data().names_repository,
        &connector,
        ctx.data().config.nicknamer.clone(),
    );
    match member {
        Some(member) => {
            nicknamer.reveal(&member.into()).await?;
            Ok(())
        }
        None => {
            nicknamer.reveal_all().await?;
            Ok(())
        }
    }
}

/// Logs message contents when a message is created
async fn on_message_create(_ctx: &serenity::Context, new_message: &Message) {
    info!("Message created: {}", new_message.content);
}

#[tokio::main]
async fn main() {
    let stdout = ConsoleAppender::builder().build();
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("nicknamer", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    let _log4rs_handle = log4rs::init_config(config).unwrap();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let framework = poise::Framework::<
        discord::serenity::Data<EmbeddedNamesRepository>,
        anyhow::Error,
    >::builder()
    .options(poise::FrameworkOptions {
        commands: vec![help(), ping(), reveal(), nick()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            ..Default::default()
        },
        event_handler: |ctx, event, _framework, _data| {
            Box::pin(async move {
                match &event {
                    FullEvent::Message { new_message } => {
                        on_message_create(ctx, new_message).await;
                    }
                    _ => debug!("Unhandled event: {:?}", event),
                }
                Ok(())
            })
        },
        ..Default::default()
    })
    .setup(|ctx, _ready, framework| {
        Box::pin(async move {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(discord::serenity::Data {
                names_repository: EmbeddedNamesRepository::new(),
                config: Config::new().expect("failed to load config"),
            })
        })
    })
    .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    info!("Starting bot...");
    client.unwrap().start().await.unwrap();
}
