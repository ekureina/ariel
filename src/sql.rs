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

use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::entities::{fic_platforms, fics, prelude::*, users};

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
