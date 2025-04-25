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

use std::{collections::HashMap, path::Path, time::SystemTime};

/// Generates a link to the given Phoenix Saga Song
#[poise::command(prefix_command, slash_command)]
pub async fn song(
    ctx: crate::PoiseContext<'_>,
    #[description = "Song to link to"] song: Option<String>,
) -> Result<(), crate::PoiseError> {
    let url = match song {
        Some(ref song) => ctx
            .data()
            .song_cache
            .data
            .get(&song.to_lowercase())
            .cloned(),
        None => Some(String::from("https://archiveofourown.org/series/3790873")),
    };
    if let Some(url) = url {
        ctx.reply(url).await?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SongCache {
    update_time: SystemTime,
    data: HashMap<String, String>,
}

impl SongCache {
    pub async fn new(path: impl AsRef<Path> + Clone) -> std::io::Result<Self> {
        let update_time = tokio::fs::metadata(path.clone()).await?.modified()?;
        let song_data = tokio::fs::read_to_string(path).await?;
        let data = serde_json::from_str(&song_data).expect("data json");
        Ok(SongCache { update_time, data })
    }
}
