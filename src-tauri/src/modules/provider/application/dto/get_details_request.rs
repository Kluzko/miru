use serde::{Deserialize, Serialize};
use specta::Type;

use crate::modules::provider::AnimeProvider;

/// Request DTO for getting anime details
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAnimeDetailsRequest {
    /// Anime ID (can be from any provider)
    pub id: String,
    /// Preferred provider to use for fetching details
    pub preferred_provider: Option<AnimeProvider>,
    /// Whether to enhance details with data from multiple providers
    pub enhance_with_multiple_providers: Option<bool>,
    /// Minimum quality threshold for the result
    pub quality_threshold: Option<f32>,
}

impl GetAnimeDetailsRequest {
    pub fn new(id: String) -> Self {
        Self {
            id,
            preferred_provider: None,
            enhance_with_multiple_providers: None,
            quality_threshold: None,
        }
    }

    pub fn with_provider(mut self, provider: AnimeProvider) -> Self {
        self.preferred_provider = Some(provider);
        self
    }

    pub fn with_enhancement(mut self, enhance: bool) -> Self {
        self.enhance_with_multiple_providers = Some(enhance);
        self
    }

    pub fn with_quality_threshold(mut self, threshold: f32) -> Self {
        self.quality_threshold = Some(threshold);
        self
    }
}
