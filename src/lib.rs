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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PoiseData {}

type PoiseError = Box<dyn std::error::Error + Send + Sync>;
type PoiseContext<'a> = poise::Context<'a, PoiseData, PoiseError>;

/// Ping RankoBot, and optionally echo the arguments
#[poise::command(prefix_command)]
pub async fn ping(
    ctx: PoiseContext<'_>,
    #[description = "Echo Text"] ping: Option<String>,
) -> Result<(), PoiseError> {
    let response = format!(
        "Pong{}!",
        ping.map(|text| format!(" {}", text))
            .unwrap_or_else(String::new)
    );
    ctx.reply(response).await?;
    Ok(())
}

/// Register RankoBot's slash commands
#[poise::command(prefix_command)]
pub async fn register(ctx: PoiseContext<'_>) -> Result<(), PoiseError> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
