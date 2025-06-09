mod nicknamer;

use self::nicknamer::config::Config;
use self::nicknamer::connectors::discord;
use self::nicknamer::connectors::discord::serenity::{Context, SerenityDiscordConnector};
use self::nicknamer::names::EmbeddedNamesRepository;
use crate::nicknamer::{Nicknamer, NicknamerImpl};
use axum::Router;
use include_dir::{Dir, include_dir};
use log::{LevelFilter, debug, info};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{FullEvent, Member, Message};

static CONFIG_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/config");

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
#[poise::command(prefix_command)]
async fn reveal(
    ctx: Context<'_>,
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

/// Logs message contents when a message is created
async fn on_message_create(_ctx: &serenity::Context, new_message: &Message) {
    info!("Message created: {}", new_message.content);
}

fn configure_logging() {
    let stdout = ConsoleAppender::builder().build();
    let log_config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("nicknamer", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    let _log4rs_handle = log4rs::init_config(log_config).unwrap();
}

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

    tokio::spawn(async {
        axum::serve(listener, app.into_make_service())
            .await
            .expect("Web server encountered an error");
    });
}

async fn configure_discord_bot() -> anyhow::Result<serenity::Client> {
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
                names_repository: EmbeddedNamesRepository::new()
                    .expect("failed to load names repository"),
                config: Config::new().expect("failed to load config"),
            })
        })
    })
    .build();

    serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .map_err(anyhow::Error::from)
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    // Section 1: Log configuration
    configure_logging();

    // Section 2: Web server start up (initiation)
    // Call start_web_server to begin its setup and get the future for its completion.
    // The actual server runs in a spawned task.
    let web_server_setup_completion_future = start_web_server();

    // Section 3: Discord bot start up
    // Configure and start the Discord bot. client.start().await is typically a long-running task.
    let client_result = configure_discord_bot().await;

    match client_result {
        Ok(mut client) => {
            if let Err(why) = client.start().await {
                // client.start().await is blocking. If it errors, the bot failed to run or stopped with an error.
                log::error!("Discord client execution failed or stopped with error: {:?}", why);
            } else {
                // If client.start() returns Ok(()), it means the bot shut down gracefully.
                log::info!("Discord bot main loop exited gracefully.");
            }
        }
        Err(e) => {
            log::error!("Failed to configure Discord bot: {:?}", e);
        }
    }
    log::info!("Discord bot startup sequence concluded.");

    // Await the web server's initial setup completion.
    // This is done after the Discord bot's startup sequence (including its main loop attempt).
    // This ensures that start_web_server() has finished its own async operations (e.g., binding).
    log::info!("Awaiting completion of web server initial setup...");
    web_server_setup_completion_future.await;
    log::info!("Web server initial setup complete. Server is running in a background task.");

    // If client.start().await was the main blocking operation and it has returned (e.g., bot stopped),
    // main will now exit. This will also lead to the termination of spawned tasks like the web server.
    // This is typical behavior for such applications.
}
