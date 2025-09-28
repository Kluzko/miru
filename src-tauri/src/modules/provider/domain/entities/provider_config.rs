use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::Duration;

use crate::modules::provider::AnimeProvider;

/// Simple provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderConfig {
    pub provider: AnimeProvider,
    pub enabled: bool,
    pub priority: u32,
    pub timeout_seconds: u32,
    pub base_url: String,
}

impl ProviderConfig {
    pub fn new(provider: AnimeProvider, enabled: bool, priority: u32) -> Self {
        let (base_url, timeout) = match provider {
            AnimeProvider::AniList => ("https://graphql.anilist.co".to_string(), 10),
            AnimeProvider::Jikan => ("https://api.jikan.moe/v4".to_string(), 8),
            AnimeProvider::Kitsu => ("https://kitsu.io/api/edge".to_string(), 8),
            AnimeProvider::TMDB => ("https://api.themoviedb.org/3".to_string(), 8),
            AnimeProvider::AniDB => ("https://anidb.net/api".to_string(), 12),
        };

        Self {
            provider,
            enabled,
            priority,
            timeout_seconds: timeout,
            base_url,
        }
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds as u64)
    }

    pub fn is_available(&self) -> bool {
        self.enabled
    }
}
