use poise::serenity_prelude::Emoji;
use rand::seq::IndexedRandom;
use sea_orm::{EntityTrait, QueryFilter, prelude::*, sea_query::Condition};
use tracing::{info, instrument};

use crate::{
    PoiseContext, PoiseError,
    entities::{fics, prelude::*},
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
/// Command to grab a random Song, that appears before a given point in `The Phoenix Saga`
#[poise::command(prefix_command, slash_command)]
#[instrument(skip(ctx))]
pub async fn random_song(
    ctx: PoiseContext<'_>,
    #[description = "Maximum Phoenix Book Number, defaults to latest book"] max_book_number: Option<
        i32,
    >,
    #[description = "Maximum Phoenix Chapter Number in book, defaults to end of book"]
    max_chapter_number: Option<i32>,
    #[description = "Fic Platform to use, defaults to AO3"] fic_platform: Option<String>,
) -> Result<(), PoiseError> {
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
            .filter(fics::Column::PlatformId.eq(platform_id))
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
            ctx.reply(emoji_header + &picked_fic.unwrap().url).await?;
        }
    } else {
        ctx.reply("Unknown Fic Platform, could not not find song")
            .await?;
    }
    Ok(())
}
