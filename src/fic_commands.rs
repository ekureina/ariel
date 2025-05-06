use poise::serenity_prelude::Emoji;
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

use crate::{
    PoiseContext, PoiseError,
    entities::{fics, prelude::*},
    sql,
};
use sea_orm::{EntityTrait, QueryFilter, prelude::*};

#[poise::command(prefix_command, slash_command)]
#[instrument(skip(ctx))]
pub async fn add_fic(
    ctx: PoiseContext<'_>,
    #[description = "Name of the pic to add"] title: String,
    #[description = "Name of the platform to add to, defaults to AO3"]
    #[autocomplete = "crate::sql::get_platform_autocomplete"]
    platform: Option<String>,
    #[description = "URL to the fic"] url: String,
    #[description = "Fandoms associated to the fic, defaults to \"Ranma 1/2\". For Crossovers, separate by commas."]
    fandoms: Option<String>,
) -> Result<(), PoiseError> {
    let db = &ctx.data().database_connection;
    let platform = platform.unwrap_or_else(|| String::from("Archive of Our Own"));
    let (platform_id, _) = sql::get_fic_platform_id_and_emoji_name_from_name(&platform, db)
        .await
        .map_err(|err| err.to_string())?
        .ok_or("Unable to find Fic Platform")?;
    let user = sql::get_user(ctx.author().id.get(), db).await?;
    let fandoms = match fandoms {
        None => vec![String::from("Ranma 1/2")],
        Some(fandoms) => fandoms
            .split(",")
            .map(|fandom| fandom.trim().to_owned())
            .collect(),
    };
    let new_fic =
        sql::insert_fic_if_not_exists(&title, platform_id, user.id, url, None, None, fandoms, db)
            .await?;
    info!("Created fic: {new_fic:?}");
    ctx.reply(String::from("Added fic ") + &title + " for " + &platform)
        .await?;
    Ok(())
}

/// Command to grab a link to a specied Fic, for the specified platform
#[poise::command(prefix_command, slash_command)]
#[instrument(skip(ctx))]
pub async fn fic(
    ctx: PoiseContext<'_>,
    #[description = "Name of the fic to link to"]
    #[autocomplete = "crate::sql::get_fic_autocomplete"]
    name: String,
    #[description = "Name of the platform to link to, defaults to AO3"]
    #[autocomplete = "crate::sql::get_platform_autocomplete"]
    platform: Option<String>,
) -> Result<(), PoiseError> {
    let db = &*ctx.data().database_connection;
    let platform = platform.unwrap_or_else(|| String::from("Archive of Our Own"));
    if let Some((platform_id, platform_emoji)) =
        sql::get_fic_platform_id_and_emoji_name_from_name(&platform, db).await?
    {
        let fic = Fics::find()
            .filter(fics::Column::PlatformId.eq(platform_id))
            .filter(fics::Column::Title.eq(&name))
            // A Fic, not a Phoenix Song
            .filter(fics::Column::PhoenixSongBook.is_null())
            .filter(fics::Column::PhoenixSongChapter.is_null())
            .one(db)
            .await?;
        let emoji = match platform_emoji {
            Some(emoji_name) => ctx
                .partial_guild()
                .await
                .map(|partial_guild| partial_guild.emojis)
                .and_then(|emojis| {
                    emojis
                        .values()
                        .find(|emoji| emoji.name == emoji_name)
                        .map(Emoji::to_string)
                }),
            None => None,
        };
        let emoji_header = emoji.map(|emoji| format!("{emoji}: ")).unwrap_or_default();
        match fic {
            Some(fic) => {
                info!("Found fic: {fic:?}");
                ctx.reply(emoji_header + &fic.url).await?;
            }
            None => {
                ctx.reply(emoji_header + "No fics called " + &name + " exist!")
                    .await?;
            }
        }
    } else {
        ctx.reply("Unknown Fic Platform, could not not find fic")
            .await?;
    }
    Ok(())
}
