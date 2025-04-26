use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use notify::Event;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{error, info, instrument};

use crate::{PoiseData, phoenix::song::SongCache};

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
#[instrument(skip_all)]
pub async fn update_song_cache(
    mut rx: UnboundedReceiver<Result<Event, notify::Error>>,
    data: Arc<Mutex<PoiseData>>,
    song_path_cache: PathBuf,
) -> Result<(), poise::serenity_prelude::Error> {
    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                info!("Found event: {event:?}");
                for path in event.paths {
                    info!("Path {path:?} updated!");
                    if path.canonicalize().unwrap() == song_path_cache.canonicalize().unwrap() {
                        let song_cache = SongCache::new(song_path_cache.clone())
                            .await
                            .expect("Unable to load song path cache");
                        data.lock().expect("Mutex not poisoned").song_cache = song_cache;
                    }
                }
            }
            Err(err) => error!("Found error: {err}"),
        }
    }
    info!("Finished processing songs");
    Ok(())
}
