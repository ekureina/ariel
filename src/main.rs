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

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use ariel::{PoiseData, phoenix::song::SongCache, tasks::update_song_cache};
use clap::Parser;

use notify::{PollWatcher, Watcher};
use poise::serenity_prelude as serenity;

use tokio::task::JoinSet;
use tracing_subscriber::{EnvFilter, prelude::*};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, clap::Parser)]
struct ArielArgs {
    /// The path to load the song cache file from
    #[arg(short, long, default_value = "./song_cache.json")]
    song_cache_path: String,
    /// The discord token to use to connect to Discord
    #[arg(short, long, env)]
    pub discord_token: String,
    /// The length of time between polling  for file updates, in ms.
    /// Defaults to one hour
    #[arg(short, long, default_value_t = 36_000)]
    pub file_watcher_poll_ms: u64,
}

impl ArielArgs {
    pub fn get_song_cache_path(&self) -> Result<PathBuf, std::io::Error> {
        fs::canonicalize(&self.song_cache_path)
    }

    pub fn get_file_watcher_poll_time(&self) -> Duration {
        Duration::from_millis(self.file_watcher_poll_ms)
    }
}

struct NotifyTokioSender(tokio::sync::mpsc::UnboundedSender<Result<notify::Event, notify::Error>>);

impl notify::EventHandler for NotifyTokioSender {
    fn handle_event(&mut self, event: notify::Result<notify::Event>) {
        let _ = self.0.send(event);
    }
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
    let data = Arc::new(Mutex::new(PoiseData::new(
        SongCache::new(
            args.get_song_cache_path()
                .expect("Expect song cache path correct"),
        )
        .await
        .expect("Unable to load Song Cache"),
    )));

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let mut song_watcher = PollWatcher::new(
        NotifyTokioSender(tx),
        notify::Config::default().with_poll_interval(args.get_file_watcher_poll_time()),
    )
    .expect("Song Cache File Watcher unable to create");

    let framework_data = data.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ariel::ping(),
                ariel::register(),
                ariel::phoenix::song::song(),
            ],
            on_error: |error| Box::pin(async move { ariel::on_error(&error) }),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(framework_data)
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
    join_set.spawn(ariel::repl::run_commands());
    song_watcher
        .watch(
            args.get_song_cache_path()
                .expect("Expect canonical song cache path")
                .as_path(),
            notify::RecursiveMode::NonRecursive,
        )
        .expect("File Watcher Can't watch");
    join_set.spawn(update_song_cache(
        rx,
        data,
        args.get_song_cache_path()
            .expect("Expect canonical song cache path"),
    ));
    join_set
        .join_next()
        .await
        .expect("Failed to join set")
        .expect("Failed to join set")
        .expect("Failed bot");
}
