use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fmt;

/// Supported anime data providers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, Hash, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::MediaProvider"]
pub enum AnimeProvider {
    /// Jikan (MyAnimeList API) - Default provider
    #[serde(rename = "jikan")]
    #[db_rename = "Jikan"]
    Jikan,
    /// AniList GraphQL API
    #[serde(rename = "anilist")]
    #[db_rename = "AniList"]
    AniList,
    /// Kitsu API
    #[serde(rename = "kitsu")]
    #[db_rename = "Kitsu"]
    Kitsu,
    /// TMDB for anime movies
    #[serde(rename = "tmdb")]
    #[db_rename = "TMDB"]
    TMDB,
    /// AniDB for detailed technical info
    #[serde(rename = "anidb")]
    #[db_rename = "AniDB"]
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
