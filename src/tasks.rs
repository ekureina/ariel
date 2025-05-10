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

use chrono::{Days, NaiveTime, Utc};
use poise::serenity_prelude::{CacheHttp, ChannelId, CreateMessage, CreateThread};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::time::sleep;
use tracing::{error, info};

use crate::sql;

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub(crate) struct WritingPrompt {
    pub pov: String,
    pub conflict: String,
    pub time: String,
    pub theme: String,
    pub place: String,
    pub character_identity: String,
    pub character_trait: String,
    pub character_background: String,
}

impl std::fmt::Display for WritingPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "POV: {}", self.pov)?;
        writeln!(f, "Conflict: {}", self.conflict)?;
        writeln!(f, "Time: {}", self.time)?;
        writeln!(f, "Theme: {}", self.theme)?;
        writeln!(f, "Place: {}", self.place)?;
        writeln!(f, "Character Identity: {}", self.character_identity)?;
        writeln!(f, "Character Trait: {}", self.character_trait)?;
        writeln!(f, "Character Background: {}", self.character_background)
    }
}

/// Task to daily create a writing dice thread, in the specified channel
/// # Errors
/// Should not error, logs instead
/// # Panics
/// Panics if not written correctly
pub async fn writing_dice_threads<CH: CacheHttp + Clone>(
    db: impl Into<Arc<DatabaseConnection>>,
    client: CH,
    channel_id: ChannelId,
    schedule_time: NaiveTime,
    time_zone: chrono_tz::Tz,
) -> Result<(), poise::serenity_prelude::Error> {
    let db = db.into();
    let mut next_set = Utc::now().with_timezone(&time_zone);
    if next_set.time() > schedule_time {
        info!("Scheduling for next day");
        next_set = next_set + Days::new(1);
    }
    next_set = next_set.with_time(schedule_time).unwrap();
    loop {
        let now = Utc::now().with_timezone(&time_zone);
        sleep(next_set.signed_duration_since(now).abs().to_std().unwrap()).await;
        next_set = next_set + Days::new(1);
        info!("Creating a daily Writing Prompt");
        let now = Utc::now().with_timezone(&time_zone);
        let prompt = sql::get_writing_prompt(&db).await.unwrap();
        let create_thread =
            CreateThread::new(String::from("Writing Dice: ") + &now.date_naive().to_string())
                .kind(poise::serenity_prelude::ChannelType::PublicThread);
        match channel_id
            .create_thread(client.clone(), create_thread)
            .await
        {
            Err(err) => error!("{err}"),
            Ok(channel) => {
                info!("Created thread!");
                let message = CreateMessage::new().content(format!("{prompt}"));
                channel.send_message(client.clone(), message).await.unwrap();
                info!("Sent message to channel");
            }
        }
    }
}
