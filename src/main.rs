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

use std::sync::Arc;

use ariel::{ArielData, migrator::Migrator};
use clap::Parser;

use poise::serenity_prelude::{self as serenity, ChannelId, RoleId, UserId};

use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tokio::task::JoinSet;
use tracing_subscriber::{EnvFilter, prelude::*};

#[derive(Clone, PartialEq, Eq, clap::Parser)]
struct ArielArgs {
    /// The discord token to use to connect to Discord
    #[arg(short, long, env)]
    pub discord_token: String,
    /// The URL to the connected SQL Database.
    /// MSQL, POSTGRES, and `SQLite` are all supported
    #[arg(long, env, default_value = "sqlite::memory:")]
    pub database_url: String,
    /// The Role Id of a special Guild Role with Admin Permissions
    #[arg(short, long, env)]
    pub admin_role_id: RoleId,
    /// The Discord User Id of the Author of The Phoenix Saga
    #[arg(long, env)]
    pub phoenix_saga_user_id: UserId,
    /// The Channel Id to post Writing Prompts to
    #[arg(long, env)]
    pub writing_dice_channel_id: ChannelId,
    /// The Timezone in which to post writing prompts
    #[arg(long, env, default_value_t = chrono_tz::Tz::America__Los_Angeles)]
    pub writing_dice_time_zone: chrono_tz::Tz,
    /// The Time each day at which to post writing prompts
    #[arg(long, env, default_value_t = chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap())]
    pub writing_dice_time: chrono::NaiveTime,
    /// The channel in which to post love spams
    #[arg(long, env)]
    pub love_spam_channel_id: ChannelId,
    /// The role users must have to get a love spam
    #[arg(long, env)]
    pub love_spam_role_id: RoleId,
    /// The Time, in the love spammee's timezone, at which to initiate a timezone
    #[arg(long,env, default_value_t = chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap())]
    pub love_spam_time: chrono::NaiveTime,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args = ArielArgs::parse();
    let db = Arc::new(
        Database::connect(&args.database_url)
            .await
            .expect("Unable to connect to database"),
    );

    // Run only unapplied migrations
    Migrator::install(&*db)
        .await
        .expect("Failed to setup migration table");
    let unapplied_count = Migrator::get_pending_migrations(&*db)
        .await
        .expect("Unable to get pending migration count")
        .len();
    match Migrator::up(&*db, None).await {
        Ok(()) => {}
        Err(err) => {
            Migrator::down(
                &*db,
                Some(
                    unapplied_count
                        .try_into()
                        .expect("Too many unapplied migrations"),
                ),
            )
            .await
            .expect("Failed to run DB Migration rollbacks");
            panic!("Unable to run migrations, rolled back: {err}")
        }
    }

    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;
    let data = ArielData::new(
        args.phoenix_saga_user_id,
        args.admin_role_id,
        Arc::clone(&db),
    );

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ariel::ping(),
                ariel::register(),
                ariel::phoenix::random_song(),
                ariel::phoenix::song(),
                ariel::phoenix::add_song(),
                ariel::fic_commands::add_fic(),
                ariel::fic_commands::fic(),
                ariel::fic_commands::random_fic(),
                ariel::add_for_birthday_lovespam(),
                ariel::remove_birthday_lovespam(),
            ],
            on_error: |error| Box::pin(async move { ariel::on_error(&error) }),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    let mut client = serenity::Client::builder(&args.discord_token, intents)
        .framework(framework)
        .await
        .expect("Err creating client");
    let http_client = ariel::TaskCacheHttp::new(client.http.clone(), client.cache.clone());

    let mut join_set = JoinSet::new();
    // Run the actual client
    join_set.spawn(async move { client.start().await });
    // Provide a means of stopping the bot without sending a kill signal
    join_set.spawn(ariel::repl::run_commands(
        db.clone(),
        args.phoenix_saga_user_id.get(),
    ));
    join_set.spawn(ariel::tasks::writing_dice_threads(
        db,
        http_client,
        args.writing_dice_channel_id,
        args.writing_dice_time,
        args.writing_dice_time_zone,
    ));
    join_set
        .join_next()
        .await
        .expect("Failed to join set")
        .expect("Failed to join set")
        .expect("Failed bot");
}
