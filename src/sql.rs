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

use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    QuerySelect, TransactionTrait,
};

use crate::{
    ArielPoiseContext,
    entities::{fandoms, fic_platforms, fics, fics_fandoms, prelude::*, urls, users},
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
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) async fn insert_fic_if_not_exists(
    title: &str,
    platform: &str,
    user_id: i32,
    url: String,
    phoenix_book: impl Into<Option<i32>>,
    phoenix_chapter: impl Into<Option<i32>>,
    fandoms: Vec<String>,
    db: &DatabaseConnection,
) -> Result<fics::Model, DbErr> {
    if let Some(fic) = Fics::find()
        .filter(fics::Column::Title.eq(title))
        .one(db)
        .await?
    {
        // Fic exists, may still need to insert platform
        match Urls::find()
            .find_also_related(FicPlatforms)
            .filter(urls::Column::FicId.eq(fic.id))
            .filter(fic_platforms::Column::Name.eq(platform))
            .one(db)
            .await?
        {
            // Fic exists with the right platform
            Some((_, Some(_))) => return Ok(fic),
            _ => {
                return match FicPlatforms::find()
                    .select_only()
                    .column(fic_platforms::Column::Id)
                    .filter(fic_platforms::Column::Name.eq(platform))
                    .into_tuple()
                    .one(db)
                    .await?
                {
                    Some(platform_id) => {
                        Urls::insert(urls::ActiveModel {
                            url: ActiveValue::Set(url),
                            fic_id: ActiveValue::Set(fic.id),
                            platform_id: ActiveValue::Set(platform_id),
                        })
                        .exec(db)
                        .await?;
                        Ok(fic)
                    }
                    None => Err(DbErr::RecordNotFound("No Platform found".to_owned())),
                };
            }
        }
    }
    let transaction = db.begin().await?;

    let fic = match Fics::insert(fics::ActiveModel {
        author_user_id: ActiveValue::Set(user_id),
        title: ActiveValue::Set(title.to_owned()),
        phoenix_song_book: ActiveValue::Set(phoenix_book.into()),
        phoenix_song_chapter: ActiveValue::Set(phoenix_chapter.into()),
        ..Default::default()
    })
    .exec_with_returning(&transaction)
    .await
    {
        Ok(fic) => fic,
        Err(err) => {
            transaction.rollback().await?;
            return Err(err);
        }
    };

    let Some(platform_id) = FicPlatforms::find()
        .select_only()
        .column(fic_platforms::Column::Id)
        .filter(fic_platforms::Column::Name.eq(platform))
        .into_tuple()
        .one(&transaction)
        .await?
    else {
        return Err(DbErr::RecordNotFound("No Platform found".to_owned()));
    };

    match Urls::insert(urls::ActiveModel {
        url: ActiveValue::Set(url),
        fic_id: ActiveValue::Set(fic.id),
        platform_id: ActiveValue::Set(platform_id),
    })
    .exec(&transaction)
    .await
    {
        Ok(url) => url,
        Err(err) => {
            transaction.rollback().await?;
            return Err(err);
        }
    };

    let mut database_fandoms = vec![];
    for fandom in fandoms {
        match get_or_create_fandom(&fandom, &transaction).await {
            Ok(fandom) => database_fandoms.push(fics_fandoms::ActiveModel {
                fic_id: ActiveValue::Set(fic.id),
                fandom_id: ActiveValue::Set(fandom.id),
            }),
            Err(err) => {
                transaction.rollback().await?;
                return Err(err);
            }
        }
    }

    match FicsFandoms::insert_many(database_fandoms)
        .exec(&transaction)
        .await
    {
        Ok(_) => {}
        Err(err) => {
            transaction.rollback().await?;
            return Err(err);
        }
    }

    transaction.commit().await?;

    Ok(fic)
}

/// Gets the specified fandom, creating if necessary
pub(crate) async fn get_or_create_fandom<C: ConnectionTrait>(
    fandom: &str,
    db: &C,
) -> Result<fandoms::Model, DbErr> {
    if let Some(fandom) = Fandoms::find()
        .filter(fandoms::Column::Name.eq(fandom))
        .one(db)
        .await?
    {
        Ok(fandom)
    } else {
        let model = fandoms::ActiveModel {
            name: ActiveValue::Set(fandom.to_owned()),
            ..Default::default()
        };
        Fandoms::insert(model).exec_with_returning(db).await
    }
}

/// Autocompletes the list of platforms ariel knows about
pub(crate) async fn get_platform_autocomplete(
    poise_ctx: ArielPoiseContext<'_>,
    partial: &str,
) -> Vec<String> {
    let db = &*poise_ctx.data().database_connection;
    FicPlatforms::find()
        .select_only()
        .column(fic_platforms::Column::Name)
        .filter(fic_platforms::Column::Name.starts_with(partial))
        .into_tuple::<String>()
        .all(db)
        .await
        .unwrap_or_default()
}

/// Autocompletes the list of fics ariel knows about
pub(crate) async fn get_fic_autocomplete(
    poise_ctx: ArielPoiseContext<'_>,
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
    poise_ctx: ArielPoiseContext<'_>,
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

/// Autocompletes the list of fandoms ariel knows about
pub(crate) async fn get_fandom_autocomplete(
    poise_ctx: ArielPoiseContext<'_>,
    partial: &str,
) -> Vec<String> {
    let db = &*poise_ctx.data().database_connection;
    let partial_owned = partial.to_owned();
    // Get the current typed in fandom, along with the prefix
    let (prefix, partial_current_fandom) = partial_owned.rsplit_once(',').map_or_else(
        || (String::new(), partial_owned.as_str()),
        |(prefix, partial_fandom)| (format!("{prefix}, "), partial_fandom),
    );

    Fandoms::find()
        .select_only()
        .column(fandoms::Column::Name)
        .filter(fandoms::Column::Name.starts_with(partial_current_fandom))
        .into_tuple::<String>()
        .all(db)
        .await
        .unwrap_or_default()
        .iter()
        .filter_map(|fandom| {
            if prefix.contains(fandom) {
                None
            } else {
                Some(prefix.clone() + fandom)
            }
        })
        .collect()
}
