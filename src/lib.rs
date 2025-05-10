use std::sync::Arc;

use entities::users;
use poise::serenity_prelude::{RoleId, UserId};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
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
pub mod fic_commands;
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

type ArielError = Box<dyn std::error::Error + Send + Sync>;
type ArielPoiseContext<'a> = poise::Context<'a, PoiseData, ArielError>;

/// Ping Ariel, and optionally echo the arguments
#[poise::command(prefix_command)]
#[instrument(skip(ctx))]
pub async fn ping(
    ctx: ArielPoiseContext<'_>,
    #[description = "Echo Text"]
    #[rest]
    ping: Option<String>,
) -> Result<(), ArielError> {
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

/// Adds a user into the database, with birthday and timezone parameters for Birthday Love Spams
#[poise::command(slash_command)]
#[instrument(skip(ctx))]
pub async fn add_for_birthday_lovespam(
    ctx: ArielPoiseContext<'_>,
    #[description = "Birthday Month"]
    #[min = 1]
    #[max = 12]
    birthday_month: u8,
    #[description = "Birthday Day"]
    #[min = 1]
    #[max = 31]
    birthday_day: u8,
    #[description = "Timezone, named (example: America/Los_Angeles)"] timezone: String,
) -> Result<(), ArielError> {
    if chrono_tz::Tz::from_str_insensitive(&timezone).is_err() {
        ctx.reply("Invalid timezone: {timezone}").await?;
    } else if chrono::NaiveDate::from_ymd_opt(2024, birthday_month.into(), birthday_day.into())
        .is_none()
    {
        // We use 2024, to allow for Leap-Day Birthdays as valid
        ctx.reply("Invalid Birthday").await?;
    } else {
        let db = &*ctx.data().database_connection;
        let user = sql::get_user(ctx.author().id.get(), db).await?;
        let user_update = users::ActiveModel {
            id: ActiveValue::Set(user.id),
            discord_id: ActiveValue::Set(user.discord_id),
            birthday_month: ActiveValue::set(Some(birthday_month.into())),
            birthday_day: ActiveValue::Set(Some(birthday_day.into())),
            time_zone: ActiveValue::Set(Some(timezone)),
            ..Default::default()
        };
        user_update.update(db).await?;
        ctx.reply(
            "Added for birthday lovespams. For regular LoveSpams, add the \"Love Spam Me\" Role.",
        )
        .await?;
    }
    Ok(())
}

/// Remove user for Birthday Love Spams
#[poise::command(slash_command)]
#[instrument(skip(ctx))]
pub async fn remove_birthday_lovespam(ctx: ArielPoiseContext<'_>) -> Result<(), ArielError> {
    let user_id = ctx.author().id.get();
    let db = &*ctx.data().database_connection;
    let user = sql::get_user(user_id, db).await?;
    let user_update = users::ActiveModel {
        id: ActiveValue::Set(user.id),
        discord_id: ActiveValue::Set(user.discord_id),
        birthday_month: ActiveValue::Set(None),
        birthday_day: ActiveValue::Set(None),
        time_zone: ActiveValue::Set(None),
        ..Default::default()
    };
    user_update.update(db).await?;

    ctx.reply("Removed from Birthday Lovespams").await?;
    Ok(())
}

/// Register Ariel's slash commands
#[poise::command(prefix_command)]
#[instrument]
pub async fn register(ctx: ArielPoiseContext<'_>) -> Result<(), ArielError> {
    info!("Registering slash commands");
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

pub fn on_error(error: &poise::FrameworkError<'_, PoiseData, ArielError>) {
    error!("Error: {}", error);
}

pub(crate) async fn is_admin(ctx: ArielPoiseContext<'_>) -> Result<bool, ArielError> {
    info!("Checking Admin permissions");
    Ok(ctx
        .author_member()
        .await
        .is_some_and(|member| member.roles.contains(&ctx.data().admin_role_id)))
}

#[allow(clippy::unused_async)]
pub(crate) async fn is_phoenix_author(ctx: ArielPoiseContext<'_>) -> Result<bool, ArielError> {
    info!("Checking if user is the Phoenix Author");
    Ok(ctx.author().id == ctx.data().phoenix_author_id)
}
