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
use tracing::{info, instrument};

use crate::{PoiseContext, PoiseError, sql};

#[poise::command(prefix_command, slash_command)]
#[instrument(skip(ctx))]
pub async fn add_fic(
    ctx: PoiseContext<'_>,
    #[description = "Name of the pic to add"] title: String,
    #[description = "Name of the platform to add to"] platform: String,
    #[description = "URL to the fic"] url: String,
) -> Result<(), PoiseError> {
    let db = &ctx.data().database_connection;
    let (platform_id, _) = sql::get_fic_platform_id_and_emoji_name_from_name(&platform, db)
        .await
        .map_err(|err| err.to_string())?
        .ok_or("Unable to find Fic Platform")?;
    let user = sql::get_user(ctx.author().id.get(), db).await?;
    let new_fic =
        sql::insert_fic_if_not_exists(&title, platform_id, user.id, url, None, None, db).await?;
    info!("Created fic: {new_fic:?}");
    ctx.reply(String::from("Added fic ") + &title + " for " + &platform)
        .await?;
    Ok(())
}
