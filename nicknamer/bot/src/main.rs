mod nicknamer;

use self::nicknamer::config::Config;
use self::nicknamer::connectors::discord;
use self::nicknamer::connectors::discord::serenity::{
    Context as PoiseContext, SerenityDiscordConnector,
};
use self::nicknamer::names::EmbeddedNamesRepository;
use crate::nicknamer::{Nicknamer, NicknamerImpl};
use anyhow::Context as AnyhowContext;
use axum::Router;
use include_dir::{Dir, include_dir};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{FullEvent, Member, Message};
use tracing::{debug, info};

static CONFIG_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/config");

/// Show this menu
#[tracing::instrument(skip(ctx))]
#[poise::command(prefix_command)]
pub async fn help(
    ctx: PoiseContext<'_>,
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

/// Ping command to test bot availability
///
/// Any instance of bot connected to the server will respond with "Pong!" and some runtime information.
#[tracing::instrument(skip(ctx))]
#[poise::command(prefix_command)]
async fn ping(ctx: PoiseContext<'_>) -> anyhow::Result<()> {
    ctx.reply("Pong!").await?;
    Ok(())
}

/// Changes the nickname for a member into a new
#[tracing::instrument(skip(ctx))]
#[poise::command(prefix_command)]
async fn nick(
    ctx: PoiseContext<'_>,
    #[description = "The specific member to reveal the name of"] member: Member,
    #[description = "The new nickname to set"] nickname: String,
) -> anyhow::Result<()> {
    let connector = SerenityDiscordConnector::new(ctx);
    let nicknamer_config = &ctx.data().config.nicknamer;
    let nicknamer = NicknamerImpl::new(&ctx.data().names_repository, &connector, nicknamer_config);
    nicknamer.change_nickname(&member.into(), &nickname).await?;
    Ok(())
}

/// Reveal members' true names, greatly diminishing their power level
///
/// Specifically, I'll reveal the names of members that can access this channel
///
/// You can also tag another member and I'll reveal the name of that person, regardless of whether they can access this channel or not
#[tracing::instrument(skip(ctx))]
#[poise::command(prefix_command)]
async fn reveal(
    ctx: PoiseContext<'_>,
    #[description = "The specific member to reveal the name of"] member: Option<Member>,
) -> anyhow::Result<()> {
    // Use the names_repository from the Data struct via the wrapper
    let connector = SerenityDiscordConnector::new(ctx);
    let nicknamer_config = &ctx.data().config.nicknamer;
    let nicknamer = NicknamerImpl::new(&ctx.data().names_repository, &connector, nicknamer_config);
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    configure_logging();

    tokio::spawn(start_web_server());

    start_discord_bot()
        .await
        .context("Discord bot failed to start or encountered a critical error during operation")?;

    Ok(())
}

fn configure_logging() {
    // The log4rs configuration is removed as we are switching to tracing.
    // If specific log4rs features like file output or complex filtering were used,
    // equivalent tracing subscribers and layers would need to be configured here.
    tracing_subscriber::fmt().init();
}

#[tracing::instrument]
async fn start_web_server() {
    let app = Router::new().route("/health", axum::routing::get(health_check));
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(3030);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind web server");
    info!("Web server running on http://{}", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Web server encountered an error");
}

async fn health_check() -> &'static str {
    "OK"
}

#[tracing::instrument]
async fn start_discord_bot() -> anyhow::Result<()> {
    info!("Initiating Discord bot startup sequence...");
    let mut client = configure_discord_bot()
        .await
        .context("Discord bot configuration failed")?;

    info!("Discord bot configured. Starting bot's main loop...");
    client
        .start()
        .await
        .context("Discord client execution failed or stopped unexpectedly")?;

    info!("Discord bot main loop exited gracefully.");
    Ok(())
}

#[tracing::instrument]
async fn configure_discord_bot() -> anyhow::Result<serenity::Client> {
    let token =
        std::env::var("DISCORD_TOKEN").context("DISCORD_TOKEN environment variable not set")?;
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
            poise::builtins::register_globally(ctx, &framework.options().commands)
                .await
                .context("Failed to register Discord commands globally")?;
            Ok(discord::serenity::Data {
                names_repository: EmbeddedNamesRepository::new()
                    .context("Failed to load embedded names repository for Discord bot")?,
                config: Config::new().context("Failed to load configuration for Discord bot")?,
            })
        })
    })
    .build();

    serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .context("Failed to create Discord client")
}

/// Logs message contents when a message is created
#[tracing::instrument(skip_all)]
async fn on_message_create(_ctx: &serenity::Context, new_message: &Message) {
    info!("Message created: {}", new_message.content);
}
