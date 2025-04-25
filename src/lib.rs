use std::sync::{Arc, Mutex};

use clap::Parser;
use phoenix::song::SongCache;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info, instrument};

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
pub mod phoenix;

/// Data held by the bot over its lifetime
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoiseData {
    song_cache: SongCache,
}

impl PoiseData {
    pub fn new(song_cache: SongCache) -> Self {
        PoiseData { song_cache }
    }
}

type PoiseError = Box<dyn std::error::Error + Send + Sync>;
type PoiseContext<'a> = poise::Context<'a, Arc<Mutex<PoiseData>>, PoiseError>;

/// Ping Ariel, and optionally echo the arguments
#[poise::command(prefix_command)]
#[instrument]
pub async fn ping(
    ctx: PoiseContext<'_>,
    #[description = "Echo Text"]
    #[rest]
    ping: Option<String>,
) -> Result<(), PoiseError> {
    let response = format!(
        "Pong{}!",
        ping.map(|text| format!(" {text}")).unwrap_or_default()
    );
    info!("Pinged Ariel, response: {response}");
    ctx.reply(response).await?;
    Ok(())
}

/// Register Ariel's slash commands
#[poise::command(prefix_command)]
#[instrument]
pub async fn register(ctx: PoiseContext<'_>) -> Result<(), PoiseError> {
    info!("Registering slash commands");
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

pub async fn on_error(error: poise::FrameworkError<'_, Arc<Mutex<PoiseData>>, PoiseError>) {
    error!("Error: {:}", error);
}

#[derive(Debug, clap::Parser)]
#[command(multicall = true)]
struct Repl {
    #[command(subcommand)]
    command: ReplCommands,
}

#[derive(Debug, clap::Subcommand)]
enum ReplCommands {
    Exit,
}

pub async fn run_commands() -> Result<(), poise::serenity_prelude::Error> {
    let mut terminal_in = BufReader::new(tokio::io::stdin());
    loop {
        let mut line = String::new();
        terminal_in.read_line(&mut line).await?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match run_command(line).await {
            Ok(quit) => {
                if quit {
                    break;
                }
            }
            Err(err) => {
                error!("Command error: {err}");
            }
        }
    }
    Ok(())
}

async fn run_command(line: &str) -> Result<bool, String> {
    let args = shlex::split(line).ok_or("failed to parse shell args")?;
    let repl = Repl::try_parse_from(args).map_err(|e| e.to_string())?;
    match repl.command {
        ReplCommands::Exit => Ok(true),
    }
}
