//! Anime title information with multiple language variants

use serde::{Deserialize, Serialize};
use specta::Type;

/// Title display preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlePreference {
    English,
    Japanese,
    Romaji,
    Native,
    Main,
}

/// Anime title information with multiple language variants
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnimeTitle {
    /// Main title (usually from the primary provider)
    pub main: String,
    /// English title
    pub english: Option<String>,
    /// Japanese title (in Japanese characters)
    pub japanese: Option<String>,
    /// Romanized Japanese title
    pub romaji: Option<String>,
    /// Native language title (could be different from Japanese for non-JP anime)
    pub native: Option<String>,
    /// Alternative titles and synonyms
    pub synonyms: Vec<String>,
}

impl AnimeTitle {
    /// Create new title with just main title
    pub fn new(main: String) -> Self {
        Self {
            main,
            english: None,
            japanese: None,
            romaji: None,
            native: None,
            synonyms: Vec::new(),
        }
    }

    /// Create title with main variants
    pub fn with_variants(
        main: String,
        english: Option<String>,
        japanese: Option<String>,
        romaji: Option<String>,
    ) -> Self {
        Self {
            main,
            english,
            japanese,
            romaji,
            native: None,
            synonyms: Vec::new(),
        }
    }
}

impl Default for AnimeTitle {
    fn default() -> Self {
        Self::new("Unknown Title".to_string())
    }
}

impl AnimeTitle {
    /// Get preferred title based on preference
    pub fn get_preferred_title(&self, preference: TitlePreference) -> &str {
        match preference {
            TitlePreference::English => self.english.as_deref().unwrap_or(&self.main),
            TitlePreference::Japanese => self.japanese.as_deref().unwrap_or(&self.main),
            TitlePreference::Romaji => self.romaji.as_deref().unwrap_or(&self.main),
            TitlePreference::Native => self.native.as_deref().unwrap_or(&self.main),
            TitlePreference::Main => &self.main,
        }
    }
}

impl std::fmt::Display for AnimeTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_preferred_title(TitlePreference::English))
    }
}
