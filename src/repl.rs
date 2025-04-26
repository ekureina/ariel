use clap::Parser;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::error;

#[derive(Debug, Parser)]
#[command(multicall = true)]
struct Repl {
    #[command(subcommand)]
    command: ReplCommands,
}

#[derive(Debug, clap::Subcommand)]
enum ReplCommands {
    Exit,
}

/// Run Ariel's Repl
/// # Errors
/// Errors if line parsing fails
pub async fn run_commands() -> Result<(), poise::serenity_prelude::Error> {
    let mut terminal_in = BufReader::new(tokio::io::stdin());
    loop {
        let mut line = String::new();
        terminal_in.read_line(&mut line).await?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match run_command(line) {
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

fn run_command(line: &str) -> Result<bool, String> {
    let args = shlex::split(line).ok_or("failed to parse shell args")?;
    let repl = Repl::try_parse_from(args).map_err(|e| e.to_string())?;
    match repl.command {
        ReplCommands::Exit => Ok(true),
    }
}
