use std::env;

use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
use tracing::error;
use tracing_subscriber::{EnvFilter, prelude::*};

/// Displays your or another user's account creation date
#[poise::command(prefix_command)]
async fn ping(
    ctx: Context<'_>,
    #[description = "Echo Text"] ping: Option<String>,
) -> Result<(), Error> {
    let response = format!(
        "Pong{}!",
        ping.map(|text| format!(" {}", text))
            .unwrap_or_else(String::new)
    );
    ctx.reply(response).await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), register()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
