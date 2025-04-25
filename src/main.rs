/*
Copyright 2025 ekureina

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::sync::{Arc, Mutex};

use ariel::{PoiseData, phoenix::song::SongCache};
use clap::Parser;

use poise::serenity_prelude as serenity;

use tokio::task::JoinSet;
use tracing_subscriber::{EnvFilter, prelude::*};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, clap::Parser)]
struct ArielArgs {
    /// The path to load the song cache file from
    #[arg(short, long, default_value = "./song_cache.json")]
    pub song_cache_path: String,
    /// The discord token to use to connect to Discord
    #[arg(short, long, env)]
    pub discord_token: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args = ArielArgs::parse();

    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ariel::ping(),
                ariel::register(),
                ariel::phoenix::song::song(),
            ],
            on_error: |error| Box::pin(ariel::on_error(error)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Arc::new(Mutex::new(PoiseData::new(
                    SongCache::new(args.song_cache_path).await?,
                ))))
            })
        })
        .build();

    let mut client = serenity::Client::builder(&args.discord_token, intents)
        .framework(framework)
        .await
        .expect("Err creating client");

    let mut join_set = JoinSet::new();
    // Run the actual client
    join_set.spawn(async move { client.start().await });
    // Provide a means of stopping the bot without sending a kill signal
    join_set.spawn(ariel::run_commands());
    join_set
        .join_next()
        .await
        .expect("Failed to join set")
        .expect("Failed to join set")
        .expect("Failed bot");
}
