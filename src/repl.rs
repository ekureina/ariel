use std::sync::Arc;

use clap::Parser;
use sea_orm::DatabaseConnection;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, instrument};

use crate::sql;

#[derive(Debug, Parser)]
#[command(multicall = true)]
struct Repl {
    #[command(subcommand)]
    command: ReplCommands,
}

#[derive(Debug, clap::Subcommand)]
enum ReplCommands {
    /// Close Ariel
    Exit,
    /// Add Phoenix Song
    AddPhoenixSong {
        /// The name of the song to add to the DB
        name: String,
        /// The Name of the Platform of the associated URL
        platform: String,
        /// The URL to the Song
        url: String,
        /// The Phoenix Book the Song Appears in
        phoenix_book: i32,
        /// The Phoenix Chapter the Song Appears in
        phoenix_chapter: i32,
    },
}

/// Run Ariel's Repl
/// # Errors
/// Errors if line parsing fails
#[instrument]
pub async fn run_commands(
    db: Arc<DatabaseConnection>,
    phoenix_saga_user_id: u64,
) -> Result<(), poise::serenity_prelude::Error> {
    let mut terminal_in = BufReader::new(tokio::io::stdin());
    loop {
        let mut line = String::new();
        if let Err(err) = terminal_in.read_line(&mut line).await {
            eprintln!("Error reading command line: {err}");
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match run_command(line, &db, phoenix_saga_user_id).await {
            Ok(quit) => {
                if quit {
                    break;
                }
            }
            Err(err) => {
                eprintln!("Command error: {err}");
            }
        }
    }
    Ok(())
}

#[instrument]
async fn run_command(
    line: &str,
    db: &DatabaseConnection,
    phoenix_saga_user_id: u64,
) -> Result<bool, String> {
    let args = shlex::split(line).ok_or("failed to parse shell args")?;
    let repl = Repl::try_parse_from(args).map_err(|e| e.to_string())?;
    match repl.command {
        ReplCommands::Exit => Ok(true),
        ReplCommands::AddPhoenixSong {
            name,
            platform,
            url,
            phoenix_book,
            phoenix_chapter,
        } => {
            let (platform_id, _) = sql::get_fic_platform_id_and_emoji_name_from_name(&platform, db)
                .await
                .map_err(|err| err.to_string())?
                .ok_or("Unable to find Fic Platform")?;
            let user = sql::get_user(phoenix_saga_user_id, db)
                .await
                .map_err(|err| err.to_string())?;
            let new_fic = sql::insert_fic_if_not_exists(
                &name,
                platform_id,
                user.id,
                url,
                phoenix_book,
                phoenix_chapter,
                db,
            )
            .await
            .map_err(|err| err.to_string())?;
            info!("Found fic: {new_fic:?}");
            Ok(false)
        }
    }
}
