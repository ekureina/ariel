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

use poise::ChoiceParameter;

use crate::Ao3Url;

/// Generates a link to the given Phoenix Saga Song
#[poise::command(prefix_command, slash_command)]
pub async fn song(
    ctx: crate::PoiseContext<'_>,
    #[description = "Song to link to"] song: Option<Song>,
) -> Result<(), crate::PoiseError> {
    if let Some(url) = song.get_url() {
        ctx.reply(url).await?;
    }
    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, poise::ChoiceParameter)]
pub(crate) enum Song {
    Rise,
    Sneak,
    #[name = "Call Me Pandora"]
    CallMePandora,
}

impl std::str::FromStr for Song {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Song::from_name(s).ok_or(())
    }
}

impl Ao3Url for Song {
    fn get_url(&self) -> Option<String> {
        match self {
            Self::Rise => Some(String::from("https://archiveofourown.org/works/50928391")),
            Self::Sneak => Some(String::from("https://archiveofourown.org/works/50928661")),
            Self::CallMePandora => Some(String::from("https://archiveofourown.org/works/51412897")),
            _ => None,
        }
    }
}

impl Ao3Url for Option<Song> {
    fn get_url(&self) -> Option<String> {
        match self {
            Some(song) => song.get_url(),
            None => Some(String::from("https://archiveofourown.org/series/3790873")),
        }
    }
}
