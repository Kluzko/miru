//! Anime provider enum and capabilities

use serde::{Deserialize, Serialize};
use specta::Type;
use std::fmt;

/// Supported anime data providers
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Hash)]
pub enum AnimeProvider {
    /// Jikan (MyAnimeList API) - Default provider
    #[serde(rename = "jikan")]
    Jikan,
    /// AniList GraphQL API
    #[serde(rename = "anilist")]
    AniList,
    /// Kitsu API
    #[serde(rename = "kitsu")]
    Kitsu,
    /// TMDB for anime movies
    #[serde(rename = "tmdb")]
    TMDB,
    /// AniDB for detailed technical info
    #[serde(rename = "anidb")]
    AniDB,
}

impl AnimeProvider {
    /// Get default provider (Jikan)
    pub fn default() -> Self {
        Self::Jikan
    }
}

impl fmt::Display for AnimeProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AnimeProvider::Jikan => "jikan",
            AnimeProvider::AniList => "anilist",
            AnimeProvider::Kitsu => "kitsu",
            AnimeProvider::TMDB => "tmdb",
            AnimeProvider::AniDB => "anidb",
        };
        write!(f, "{}", name)
    }
}

// ProviderFeature enum removed - was unused
// If needed in future, can be re-added with specific features required
