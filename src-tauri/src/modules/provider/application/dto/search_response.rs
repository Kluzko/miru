use serde::{Deserialize, Serialize};
use specta::Type;

use crate::modules::provider::{domain::AnimeData, AnimeProvider};

/// Response DTO for anime search
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchAnimeResponse {
    /// Found anime results
    pub results: Vec<AnimeData>,
    /// Total number of results found
    pub total_found: usize,
    /// Time taken for the search in milliseconds
    pub search_duration_ms: u64,
    /// Providers that were used in the search
    pub providers_used: Vec<AnimeProvider>,
    /// Quality threshold that was applied
    pub quality_threshold: f32,
}

impl SearchAnimeResponse {
    /// Get results sorted by quality score (highest first)
    pub fn results_by_quality(&self) -> Vec<AnimeData> {
        let mut sorted = self.results.clone();
        sorted.sort_by(|a, b| {
            b.quality
                .score
                .partial_cmp(&a.quality.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Get results that meet a specific quality threshold
    pub fn results_above_threshold(&self, threshold: f32) -> Vec<AnimeData> {
        self.results
            .iter()
            .filter(|anime| anime.quality.score >= threshold)
            .cloned()
            .collect()
    }

    /// Get average quality score of all results
    pub fn average_quality_score(&self) -> f32 {
        if self.results.is_empty() {
            return 0.0;
        }

        let total_score: f32 = self.results.iter().map(|anime| anime.quality.score).sum();
        total_score / self.results.len() as f32
    }

    /// Check if search was successful (found any results)
    pub fn is_successful(&self) -> bool {
        !self.results.is_empty()
    }

    /// Get search performance category
    pub fn performance_category(&self) -> &'static str {
        match self.search_duration_ms {
            0..=500 => "Excellent",
            501..=1500 => "Good",
            1501..=3000 => "Fair",
            _ => "Slow",
        }
    }
}
