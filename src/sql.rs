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

use std::collections::HashMap;

use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::{
    PoiseContext,
    entities::{fic_platforms, fics, prelude::*, users},
};

/// Helper method to translate a platform name into an id and emoji
pub(crate) async fn get_fic_platform_id_and_emoji_name_from_name(
    name: &str,
    db: &DatabaseConnection,
) -> Result<Option<(i32, Option<String>)>, DbErr> {
    Ok(FicPlatforms::find()
        .filter(fic_platforms::Column::Name.eq(name))
        .one(db)
        .await?
        .map(|platform| (platform.id, platform.emoji)))
}

/// Gets Data on the `User` with the given Discord User Id
/// Creates this user, if they do not exist in the database
pub(crate) async fn get_user(
    discord_user_id: u64,
    db: &DatabaseConnection,
) -> Result<users::Model, DbErr> {
    if let Some(user) = Users::find()
        .filter(users::Column::DiscordId.eq(discord_user_id.to_string()))
        .one(db)
        .await?
    {
        Ok(user)
    } else {
        let model = users::ActiveModel {
            discord_id: ActiveValue::Set(discord_user_id.to_string()),
            ..Default::default()
        };
        Users::insert(model).exec_with_returning(db).await
    }
}

/// Helper method to add in a fic to the database
/// Does not support Rocktails
///
/// If a Fic exists with the given title for the given platform, performs a no-op,
/// even if this is setting a different URL on User Id
pub(crate) async fn insert_fic_if_not_exists(
    title: &str,
    platform_id: i32,
    user_id: i32,
    url: String,
    phoenix_book: impl Into<Option<i32>>,
    phoenix_chapter: impl Into<Option<i32>>,
    db: &DatabaseConnection,
) -> Result<fics::Model, DbErr> {
    if let Some(fic) = Fics::find()
        .filter(fics::Column::PlatformId.eq(platform_id))
        .filter(fics::Column::Title.eq(title))
        .one(db)
        .await?
    {
        Ok(fic)
    } else {
        let model = fics::ActiveModel {
            author_user_id: ActiveValue::Set(user_id),
            platform_id: ActiveValue::Set(platform_id),
            title: ActiveValue::Set(title.to_owned()),
            url: ActiveValue::Set(url),
            phoenix_song_book: ActiveValue::Set(phoenix_book.into()),
            phoenix_song_chapter: ActiveValue::Set(phoenix_chapter.into()),
            ..Default::default()
        };

        Fics::insert(model).exec_with_returning(db).await
    }
}

/// Autocompletes the list of platforms ariel knows about
pub(crate) async fn get_platform_autocomplete(
    poise_ctx: PoiseContext<'_>,
    partial: &str,
) -> Vec<String> {
    let db = &*poise_ctx.data().database_connection;
    FicPlatforms::find()
        .filter(fic_platforms::Column::Name.starts_with(partial))
        .all(db)
        .await
        .map(|platforms| {
            platforms
                .iter()
                .map(|platform| platform.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// Autocompletes the list of fics ariel knows about
pub(crate) async fn get_fic_autocomplete(
    poise_ctx: PoiseContext<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    let db = &*poise_ctx.data().database_connection;
    let fics_with_platforms = Fics::find()
        .find_also_related(FicPlatforms)
        .filter(fics::Column::Title.starts_with(partial))
        .filter(fics::Column::PhoenixSongBook.is_null())
        .filter(fics::Column::PhoenixSongChapter.is_null())
        .all(db)
        .await
        .unwrap_or_default();

    let mut fics_to_platforms =
        HashMap::<String, Vec<String>, _>::with_capacity(fics_with_platforms.len());

    for (fic, platform) in fics_with_platforms {
        if let Some(platform) = platform {
            fics_to_platforms
                .entry(fic.title)
                .and_modify(|platforms| platforms.push(platform.name.clone()))
                .or_insert_with(|| vec![platform.name]);
        }
    }

    fics_to_platforms.into_iter().map(|(fic, platforms)| {
        poise::serenity_prelude::AutocompleteChoice::new(
            format!("{fic} ({})", platforms.join(", ")),
            fic,
        )
    })
}

/// Autocompletes the list of songs ariel knows about
pub(crate) async fn get_song_autocomplete(
    poise_ctx: PoiseContext<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    let db = &*poise_ctx.data().database_connection;
    let fics_with_platforms = Fics::find()
        .find_also_related(FicPlatforms)
        .filter(fics::Column::Title.starts_with(partial))
        .filter(fics::Column::PhoenixSongBook.is_not_null())
        .filter(fics::Column::PhoenixSongChapter.is_not_null())
        .all(db)
        .await
        .unwrap_or_default();

    let mut fics_to_platforms =
        HashMap::<String, Vec<String>, _>::with_capacity(fics_with_platforms.len());

    for (fic, platform) in fics_with_platforms {
        if let Some(platform) = platform {
            fics_to_platforms
                .entry(fic.title)
                .and_modify(|platforms| platforms.push(platform.name.clone()))
                .or_insert_with(|| vec![platform.name]);
        }
    }

    fics_to_platforms.into_iter().map(|(fic, platforms)| {
        poise::serenity_prelude::AutocompleteChoice::new(
            format!("{fic} ({})", platforms.join(", ")),
            fic,
        )
    })
}
