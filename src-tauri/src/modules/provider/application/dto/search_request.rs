use serde::{Deserialize, Serialize};
use specta::Type;

use crate::modules::provider::AnimeProvider;

/// Request DTO for anime search
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchAnimeRequest {
    /// Search query string
    pub query: String,
    /// Maximum number of results to return
    pub limit: Option<usize>,
    /// Minimum quality threshold (0.0 to 1.0)
    pub quality_threshold: Option<f32>,
    /// Whether to enhance results with data from multiple providers
    pub enhance_with_multiple_providers: Option<bool>,
    /// Preferred providers in order of preference
    pub preferred_providers: Option<Vec<AnimeProvider>>,
}

impl SearchAnimeRequest {
    pub fn new(query: String) -> Self {
        Self {
            query,
            limit: None,
            quality_threshold: None,
            enhance_with_multiple_providers: None,
            preferred_providers: None,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_quality_threshold(mut self, threshold: f32) -> Self {
        self.quality_threshold = Some(threshold);
        self
    }

    pub fn with_enhancement(mut self, enhance: bool) -> Self {
        self.enhance_with_multiple_providers = Some(enhance);
        self
    }

    pub fn with_preferred_providers(mut self, providers: Vec<AnimeProvider>) -> Self {
        self.preferred_providers = Some(providers);
        self
    }
}
