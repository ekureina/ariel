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

use chrono::{Datelike, Days, Local, Months, NaiveTime, TimeDelta, TimeZone, Utc};
use poise::serenity_prelude::{
    CacheHttp, ChannelId, CreateMessage, CreateThread, GuildId, Mentionable, RoleId,
    futures::StreamExt,
};
use rand::seq::IndexedRandom;
use sea_orm::DatabaseConnection;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing::{error, info};

use crate::{entities::users, sql};

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

/// Task to daily create a writing dice thread, in the specified channel
/// # Errors
/// Should not error, logs instead
/// # Panics
/// Panics if not written correctly
pub async fn love_spam_threads<CH: CacheHttp + Clone + 'static>(
    db: impl Into<Arc<DatabaseConnection>>,
    client: CH,
    guild_id: GuildId,
    love_spam_role_id: RoleId,
    love_spammers_role_id: RoleId,
    channel_id: ChannelId,
) -> Result<(), poise::serenity_prelude::Error> {
    let db = &*db.into();
    loop {
        let spammable_members = guild_id
            .members_iter(client.http())
            .filter_map(|member_result| async move {
                member_result
                    .ok()
                    .filter(|member| member.roles.contains(&love_spam_role_id))
            })
            // TODO: Limit the number of SQL calls here
            .filter_map(|member| async move { sql::get_user(member.user.id.get(), db).await.ok() })
            .collect::<Vec<_>>()
            .await;
        if spammable_members.is_empty() {
            info!("No members to create a love spam for!");
            // Wait an hour, and check again
            sleep(Duration::from_hours(1)).await;
            continue;
        }

        let (users, is_birthday) = next_love_spam(spammable_members).await;
        for user in users {
            let tomorrow = Local::now().checked_add_days(Days::new(1)).unwrap();
            let thread_creation_instant = match user.time_zone.as_ref() {
                Some(time_zone) => chrono_tz::Tz::from_str_insensitive(&time_zone)
                    .expect("Checked on entry to database")
                    .with_ymd_and_hms(tomorrow.year(), tomorrow.month(), tomorrow.day(), 12, 0, 0)
                    .earliest()
                    .unwrap()
                    .with_timezone(&Local),
                None => tomorrow
                    .with_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
                    .unwrap(),
            };

            let client_clone = client.clone();
            tokio::spawn(async move {
                let sleep_time = thread_creation_instant
                    .signed_duration_since(Local::now())
                    .max(TimeDelta::zero())
                    .to_std()
                    .expect("Should be at least zero duration");
                info!("{user:?}: {is_birthday}");
                sleep(sleep_time).await;
                let spam_member = guild_id
                    .member(&client_clone, user.discord_id.parse::<u64>().unwrap())
                    .await
                    .unwrap();
                let create_thread = CreateThread::new(spam_member.mention().to_string())
                    .kind(poise::serenity_prelude::ChannelType::PublicThread);
                match channel_id.create_thread(&client_clone, create_thread).await {
                    Err(err) => error!("{err}"),
                    Ok(channel) => {
                        info!("Created thread!");
                        let message_text = if is_birthday {
                            format!(
                                "Hey {}! You're never gonna believe what day it is! It's {}'s birthday! Help join me in making their day just a little bit brighter!",
                                love_spammers_role_id.mention(),
                                spam_member.mention()
                            )
                        } else {
                            format!(
                                "Hey {}! It's time to spread the love, {} looked like they could use some!",
                                love_spammers_role_id.mention(),
                                spam_member.mention()
                            )
                        };
                        let message = CreateMessage::new().content(message_text);
                        channel.send_message(&client_clone, message).await.unwrap();
                        info!("Sent message to channel");
                    }
                }
            });
        }

        sleep(Duration::from_hours(24)).await;
    }
}

async fn next_love_spam(spammable_members: Vec<users::Model>) -> (Vec<users::Model>, bool) {
    let mut rng = rand::rng();
    let tomorrow = Local::now().checked_add_days(Days::new(1)).unwrap();
    let mut birthday_users = vec![];
    for spammable_member in &spammable_members {
        match (
            spammable_member.birthday_month,
            spammable_member.birthday_day,
            spammable_member.time_zone.as_ref(),
        ) {
            (Some(month), Some(day), Some(time_zone)) => {
                let tz = chrono_tz::Tz::from_str_insensitive(&time_zone)
                    .expect("Checked on entry to database");
                let year_offset: u32 =
                    u32::try_from(tomorrow.year()).expect("Should be a year after 2024") - 2024;
                let birthday_this_year = tz
                    .with_ymd_and_hms(
                        2024,
                        month
                            .try_into()
                            .expect("Checked when entered into database"),
                        day.try_into().expect("Checked when input into database"),
                        12,
                        0,
                        0,
                    )
                    .map(|birthday| {
                        birthday
                            .checked_add_months(Months::new(year_offset * 12))
                            .expect("Adding years okay")
                            // Convert to the current time zone
                            .with_timezone(&tomorrow.timezone())
                    })
                    .earliest()
                    .expect("Should not be in a gap in time");
                if birthday_this_year.month() == tomorrow.month()
                    && birthday_this_year.day() == tomorrow.day()
                {
                    birthday_users.push(spammable_member.clone())
                }
            }
            _ => {}
        }
    }
    if !birthday_users.is_empty() {
        (birthday_users, true)
    } else {
        (
            vec![
                spammable_members
                    .choose(&mut rng)
                    .expect("At least one member available")
                    .clone(),
            ],
            false,
        )
    }
}
