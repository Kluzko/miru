use serde::{Deserialize, Serialize};
use specta::Type;

use crate::modules::provider::AnimeProvider;

/// Preferred language for search results and title matching
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum PreferredLanguage {
    /// Prioritize English titles
    English,
    /// Prioritize romanized Japanese titles
    Romaji,
    /// Prioritize native Japanese titles
    Japanese,
    /// Auto-detect language from query
    Auto,
}

/// Search criteria for anime queries
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchCriteria {
    pub query: String,
    pub preferred_language: PreferredLanguage,
    pub limit: usize,
    pub preferred_providers: Vec<AnimeProvider>,
    pub quality_threshold: f32,
    pub similarity_threshold: f64,
    pub timeout_seconds: u32,
}

impl SearchCriteria {
    pub fn new(query: String) -> Self {
        Self {
            preferred_language: Self::detect_language(&query),
            query,
            limit: 20,
            preferred_providers: vec![AnimeProvider::Jikan, AnimeProvider::AniList],
            quality_threshold: 0.7,
            similarity_threshold: 0.8,
            timeout_seconds: 10,
        }
    }

    /// Detect the likely language of the search query
    pub fn detect_language(query: &str) -> PreferredLanguage {
        // Simple language detection
        if query.chars().any(|c| c > '\u{3040}' && c < '\u{30A0}') || // Hiragana
           query.chars().any(|c| c > '\u{30A0}' && c < '\u{3100}') || // Katakana
           query.chars().any(|c| c > '\u{4E00}' && c < '\u{9FAF}')
        {
            // CJK
            PreferredLanguage::Japanese
        } else if query.chars().all(|c| c.is_ascii()) {
            PreferredLanguage::English
        } else {
            PreferredLanguage::Auto
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_providers(mut self, providers: Vec<AnimeProvider>) -> Self {
        self.preferred_providers = providers;
        self
    }

    pub fn with_quality_threshold(mut self, threshold: f32) -> Self {
        self.quality_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.query.trim().is_empty() {
            return Err("Search query cannot be empty".to_string());
        }

        if self.limit == 0 || self.limit > 100 {
            return Err("Limit must be between 1 and 100".to_string());
        }

        if self.preferred_providers.is_empty() {
            return Err("At least one provider must be specified".to_string());
        }

        Ok(())
    }
}

/// Criteria for getting anime details by ID
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetDetailsCriteria {
    pub id: String,
    pub provider: Option<AnimeProvider>,
    pub quality_threshold: f32,
    pub timeout_seconds: u32,
}

impl GetDetailsCriteria {
    pub fn new(id: String) -> Self {
        Self {
            id,
            provider: None,
            quality_threshold: 0.6,
            timeout_seconds: 8,
        }
    }

    pub fn with_provider(mut self, provider: AnimeProvider) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("ID cannot be empty".to_string());
        }

        Ok(())
    }
}
