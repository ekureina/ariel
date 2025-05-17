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
    ArielError, ArielPoiseContext,
    entities::{fandoms, fics, prelude::*, urls},
    sql,
};
use sea_orm::{
    EntityTrait, FromQueryResult, QueryFilter, QuerySelect, QueryTrait, prelude::*, sea_query::Func,
};

#[poise::command(prefix_command, slash_command)]
#[instrument(skip(ctx))]
pub async fn add_fic(
    ctx: ArielPoiseContext<'_>,
    #[description = "Name of the pic to add"] title: String,
    #[description = "Name of the platform to add to, defaults to AO3"]
    #[autocomplete = "crate::sql::get_platform_autocomplete"]
    platform: Option<String>,
    #[description = "URL to the fic"] url: String,
    #[description = "Fandoms associated to the fic, defaults to \"Ranma 1/2\". For Crossovers, separate by commas."]
    #[autocomplete = "crate::sql::get_fandom_autocomplete"]
    fandoms: Option<String>,
) -> Result<(), ArielError> {
    let db = &ctx.data().database_connection;
    let platform = platform.unwrap_or_else(|| String::from("Archive of Our Own"));
    let user = sql::get_user(ctx.author().id.get(), db).await?;
    let fandoms = match fandoms {
        None => vec![String::from("Ranma 1/2")],
        Some(fandoms) => fandoms
            .split(',')
            .map(|fandom| fandom.trim().to_owned())
            .collect(),
    };
    let new_fic =
        sql::insert_fic_if_not_exists(&title, &platform, user.id, url, None, None, fandoms, db)
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
    ctx: ArielPoiseContext<'_>,
    #[description = "Name of the fic to link to"]
    #[autocomplete = "crate::sql::get_fic_autocomplete"]
    name: String,
    #[description = "Name of the platform to link to, defaults to AO3"]
    #[autocomplete = "crate::sql::get_platform_autocomplete"]
    platform: Option<String>,
) -> Result<(), ArielError> {
    let db = &*ctx.data().database_connection;
    let platform = platform.unwrap_or_else(|| String::from("Archive of Our Own"));
    if let Some((platform_id, platform_emoji)) =
        sql::get_fic_platform_id_and_emoji_name_from_name(&platform, db).await?
    {
        let fic = Fics::find()
            .find_also_related(Urls)
            .filter(urls::Column::PlatformId.eq(platform_id))
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
            Some((fic, Some(url))) => {
                info!("Found fic: {fic:?}; {url:?}");
                ctx.reply(emoji_header + &url.url).await?;
            }
            Some((_, None)) => {
                ctx.reply("Found fic, but not for that platform.").await?;
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

/// Command to get a random fic from the given fandoms
#[poise::command(prefix_command, slash_command)]
#[instrument(skip(ctx))]
pub async fn random_fic(
    ctx: ArielPoiseContext<'_>,
    #[description = "Fandom of fic, if any. Will include crossovers into this fandom. Default will search all fandoms."]
    #[autocomplete = "crate::sql::get_single_fandom_autocomplete"]
    fandom: Option<String>,
    #[description = "Name of the platform to link to, defaults to AO3"]
    #[autocomplete = "crate::sql::get_platform_autocomplete"]
    platform: Option<String>,
) -> Result<(), ArielError> {
    #[derive(sea_orm::FromQueryResult, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    struct FicUrl {
        url: String,
    }

    let db = &*ctx.data().database_connection;
    let platform = platform.unwrap_or_else(|| String::from("Archive of Our Own"));
    let Some((platform_id, platform_emoji)) =
        sql::get_fic_platform_id_and_emoji_name_from_name(&platform, db).await?
    else {
        ctx.reply("Unable to find Fic Platform").await?;
        return Ok(());
    };

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

    let random_fic_query = Fics::find()
        .find_also_related(Fandoms)
        .find_also_related(Urls)
        .select_only()
        .column(urls::Column::Url)
        .filter(urls::Column::PlatformId.eq(platform_id))
        .filter(fics::Column::PhoenixSongBook.is_null())
        .filter(fics::Column::PhoenixSongChapter.is_null())
        // Filter fandom if provided
        .apply_if(fandom, |query, fandom_name| {
            query.filter(fandoms::Column::Name.eq(fandom_name))
        })
        .into_query()
        .order_by_expr(Func::random().into(), sea_orm::Order::Asc)
        .limit(1)
        .to_owned();

    let fic_url = FicUrl::find_by_statement(db.get_database_backend().build(&random_fic_query))
        .one(db)
        .await
        .unwrap_or_default();
    if let Some(fic_url) = fic_url {
        info!("Picked fic: {fic_url:?}");
        ctx.reply(emoji_header + &fic_url.url).await?;
    } else {
        ctx.reply(emoji_header + "No songs meet that criteria!")
            .await?;
    }
    Ok(())
}
