use std::sync::Arc;

use poise::serenity_prelude::{RoleId, UserId};
use sea_orm::DatabaseConnection;
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
pub mod entities;
pub mod migrator;
pub mod phoenix;
pub mod repl;
pub mod sql;
pub mod tasks;

/// Data held by the bot over its lifetime
#[derive(Debug, Clone)]
pub struct PoiseData {
    pub(crate) phoenix_author_id: UserId,
    pub(crate) admin_role_id: RoleId,
    pub(crate) database_connection: Arc<DatabaseConnection>,
}

impl PoiseData {
    #[must_use]
    pub fn new(
        phoenix_author_id: UserId,
        admin_role_id: RoleId,
        database_connection: impl Into<Arc<DatabaseConnection>>,
    ) -> Self {
        PoiseData {
            phoenix_author_id,
            admin_role_id,
            database_connection: database_connection.into(),
        }
    }
}

type PoiseError = Box<dyn std::error::Error + Send + Sync>;
type PoiseContext<'a> = poise::Context<'a, PoiseData, PoiseError>;

/// Ping Ariel, and optionally echo the arguments
#[poise::command(prefix_command)]
#[instrument(skip(ctx))]
pub async fn ping(
    ctx: PoiseContext<'_>,
    #[description = "Echo Text"]
    #[rest]
    ping: Option<String>,
) -> Result<(), PoiseError> {
    let is_admin_tag = if is_admin(ctx).await.unwrap_or_default() {
        String::from("from admin")
    } else {
        String::from("from regular user")
    };

    let response = format!(
        "Pong {is_admin_tag}{}!",
        ping.map(|text| format!(": {text}")).unwrap_or_default()
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

pub fn on_error(error: &poise::FrameworkError<'_, PoiseData, PoiseError>) {
    error!("Error: {:}", error);
}

pub(crate) async fn is_admin(ctx: PoiseContext<'_>) -> Result<bool, PoiseError> {
    info!("Checking Admin permissions");
    Ok(ctx
        .author_member()
        .await
        .is_some_and(|member| member.roles.contains(&ctx.data().admin_role_id)))
}

pub(crate) async fn is_phoenix_author(ctx: PoiseContext<'_>) -> Result<bool, PoiseError> {
    info!("Checking if user is the Phoenix Author");
    Ok(ctx.author().id == ctx.data().phoenix_author_id)
}
