use poise::serenity_prelude::Emoji;
use rand::seq::IndexedRandom;
use sea_orm::{EntityTrait, QueryFilter, prelude::*, sea_query::Condition};
use tracing::{info, instrument};

use crate::{
    ArielError, ArielPoiseContext,
    entities::{fics, prelude::*, urls},
    sql,
};

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

/// Adds a song for the given platform
#[poise::command(prefix_command, slash_command, check = "crate::is_phoenix_author")]
#[instrument(skip(ctx))]
pub async fn add_song(
    ctx: ArielPoiseContext<'_>,
    #[description = "Name of the song to add"] title: String,
    #[description = "Name of the platform to add to"]
    #[autocomplete = "crate::sql::get_platform_autocomplete"]
    platform: String,
    #[description = "Song's URL"] url: String,
    #[description = "Phoenix Book"] phoenix_book: i32,
    #[description = "Phoenix Chapter"] phoenix_chapter: i32,
) -> Result<(), ArielError> {
    let db = &ctx.data().database_connection;
    let phoenix_saga_user_id = ctx.data().phoenix_author_id.get();
    let user = sql::get_user(phoenix_saga_user_id, db).await?;
    let new_fic = sql::insert_fic_if_not_exists(
        &title,
        &platform,
        user.id,
        url,
        phoenix_book,
        phoenix_chapter,
        vec![String::from("Original Work"), String::from("Ranma 1/2")],
        db,
    )
    .await?;
    info!("Created song: {new_fic:?}");
    ctx.reply(String::from("Added song ") + &title + " for " + &platform)
        .await?;
    Ok(())
}

/// Command to grab a link to a specified `The Phoenx Saga` Song, for the specified platform
#[poise::command(prefix_command, slash_command)]
#[instrument(skip(ctx))]
pub async fn song(
    ctx: ArielPoiseContext<'_>,
    #[description = "Name of the song to link to"]
    #[autocomplete = "crate::sql::get_song_autocomplete"]
    title: String,
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
            .filter(fics::Column::Title.eq(&title))
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
                info!("Found fic: {fic:?}");
                ctx.reply(emoji_header + &url.url).await?;
            }
            Some((_, None)) => {
                info!("Found fic, but not the platform");
                ctx.reply("Found fic, but it does not exist for this platform")
                    .await?;
            }
            None => {
                ctx.reply(emoji_header + "No songs called " + &title + " exist!")
                    .await?;
            }
        }
    } else {
        ctx.reply("Unknown Fic Platform, could not not find song")
            .await?;
    }
    Ok(())
}

/// Command to grab a random Song, that appears before a given point in `The Phoenix Saga`
#[poise::command(prefix_command, slash_command)]
#[instrument(skip(ctx))]
pub async fn random_song(
    ctx: ArielPoiseContext<'_>,
    #[description = "Maximum Phoenix Book Number, defaults to latest book"] max_book_number: Option<
        i32,
    >,
    #[description = "Maximum Phoenix Chapter Number in book, defaults to end of book"]
    max_chapter_number: Option<i32>,
    #[description = "Fic Platform to use, defaults to AO3"]
    #[autocomplete = "crate::sql::get_platform_autocomplete"]
    fic_platform: Option<String>,
) -> Result<(), ArielError> {
    // pull out the relevant data
    let db = &*ctx.data().database_connection;
    let max_book = max_book_number.unwrap_or(i32::MAX);
    let max_chapter = max_chapter_number.unwrap_or(i32::MAX);
    let platform = fic_platform.unwrap_or(String::from("Archive of Our Own"));

    if let Some((platform_id, platform_emoji)) =
        sql::get_fic_platform_id_and_emoji_name_from_name(&platform, db).await?
    {
        // Grab all fics we could consider
        let valid_fics = Fics::find()
            .find_also_related(Urls)
            .filter(urls::Column::PlatformId.eq(platform_id))
            // Is a Phoenix song
            .filter(fics::Column::PhoenixSongBook.is_not_null())
            .filter(fics::Column::PhoenixSongChapter.is_not_null())
            // Is either in a book preceeding the max book,
            // or is in a chapter of the max book preceeding the max chapter
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(fics::Column::PhoenixSongBook.eq(max_book))
                            .add(fics::Column::PhoenixSongChapter.lte(max_chapter)),
                    )
                    .add(fics::Column::PhoenixSongBook.lt(max_book)),
            )
            .all(db)
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
        if valid_fics.is_empty() {
            ctx.reply(emoji_header + "No songs meet that criteria!")
                .await?;
        } else {
            // Expire `rng` so it isn't held across `await`
            let picked_fic = {
                let mut rng = rand::rng();
                valid_fics.choose(&mut rng)
            };
            info!("Picked fic: {picked_fic:?}");
            if let Some((_, Some(url))) = picked_fic {
                ctx.reply(emoji_header + &url.url).await?;
            }
        }
    } else {
        ctx.reply("Unknown Fic Platform, could not not find song")
            .await?;
    }
    Ok(())
}
