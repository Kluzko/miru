use serde::{Deserialize, Serialize};
use specta::Type;

use crate::modules::provider::{domain::AnimeData, AnimeProvider};

/// Response DTO for getting anime details
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAnimeDetailsResponse {
    /// The anime details if found
    pub anime: Option<AnimeData>,
    /// Time taken to fetch details in milliseconds
    pub fetch_duration_ms: u64,
    /// Provider that was actually used
    pub provider_used: Option<AnimeProvider>,
    /// Whether the anime was found
    pub found: bool,
}

impl GetAnimeDetailsResponse {
    /// Check if the response contains high-quality data
    pub fn is_high_quality(&self, threshold: f32) -> bool {
        self.anime
            .as_ref()
            .map(|anime| anime.quality.score >= threshold)
            .unwrap_or(false)
    }

    /// Get the quality score of the anime data
    pub fn quality_score(&self) -> Option<f32> {
        self.anime.as_ref().map(|anime| anime.quality.score)
    }

    /// Get fetch performance category
    pub fn performance_category(&self) -> &'static str {
        match self.fetch_duration_ms {
            0..=300 => "Excellent",
            301..=1000 => "Good",
            1001..=2000 => "Fair",
            _ => "Slow",
        }
    }

    /// Get a summary of the operation
    pub fn summary(&self) -> String {
        if self.found {
            if let Some(anime) = &self.anime {
                format!(
                    "Found '{}' with {:.1}% quality in {}ms",
                    anime.anime.title.main,
                    anime.quality.score * 100.0,
                    self.fetch_duration_ms
                )
            } else {
                format!("Found anime in {}ms", self.fetch_duration_ms)
            }
        } else {
            format!("Anime not found after {}ms", self.fetch_duration_ms)
        }
    }
}
