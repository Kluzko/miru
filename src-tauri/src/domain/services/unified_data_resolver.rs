use crate::domain::{
    entities::AnimeDetailed,
    value_objects::{AnimeProvider, UnifiedAgeRestriction},
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Provider-specific raw data for cross-provider resolution
#[derive(Debug, Clone)]
pub struct ProviderAnimeData {
    pub provider: AnimeProvider,
    pub score: Option<f32>,
    pub favorites: Option<u32>,
    pub age_restriction: Option<UnifiedAgeRestriction>,
    pub last_synced: Option<DateTime<Utc>>,
}

/// Unified data resolver for cross-provider data merging and conflict resolution
/// Implements DDD pattern for complex domain logic
pub struct UnifiedDataResolver {
    /// Priority order for age rating resolution (most detailed first)
    age_rating_priority: Vec<AnimeProvider>,
    /// Provider reliability weights for score calculation
    provider_weights: HashMap<AnimeProvider, f32>,
}

impl Default for UnifiedDataResolver {
    fn default() -> Self {
        let mut provider_weights = HashMap::new();
        provider_weights.insert(AnimeProvider::Jikan, 0.4); // Detailed ratings, large userbase
        provider_weights.insert(AnimeProvider::AniDB, 0.3); // Comprehensive data
        provider_weights.insert(AnimeProvider::AniList, 0.2); // Good community engagement
        provider_weights.insert(AnimeProvider::Kitsu, 0.1); // Basic data

        Self {
            age_rating_priority: vec![
                AnimeProvider::Jikan,   // Most detailed age ratings (G, PG, PG-13, R, etc.)
                AnimeProvider::AniDB,   // Often has detailed content warnings
                AnimeProvider::Kitsu,   // Moderate detail
                AnimeProvider::AniList, // Least detailed (boolean isAdult)
            ],
            provider_weights,
        }
    }
}

impl UnifiedDataResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve age restriction using priority-based approach
    /// Returns the most detailed age restriction available from providers
    pub fn resolve_age_restriction(
        &self,
        provider_data: &[ProviderAnimeData],
    ) -> Option<UnifiedAgeRestriction> {
        for preferred_provider in &self.age_rating_priority {
            if let Some(data) = provider_data
                .iter()
                .find(|d| d.provider == *preferred_provider)
            {
                if let Some(rating) = &data.age_restriction {
                    return Some(rating.clone());
                }
            }
        }

        // Fallback to any available rating
        provider_data.iter().find_map(|d| d.age_restriction.clone())
    }

    /// Resolve unified score using weighted average across providers
    /// Ensures consistent 0-10 scale regardless of provider
    pub fn resolve_unified_score(&self, provider_data: &[ProviderAnimeData]) -> Option<f32> {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for data in provider_data {
            if let Some(score) = data.score {
                let weight = self.provider_weights.get(&data.provider).unwrap_or(&0.1);

                // Ensure score is in 0-10 range (normalize if needed)
                let normalized_score = self.normalize_score_to_10_scale(score, &data.provider);

                weighted_sum += normalized_score * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            Some((weighted_sum / total_weight * 100.0).round() / 100.0) // Round to 2 decimals
        } else {
            None
        }
    }

    /// Resolve unified favorites/engagement metric
    /// Combines different engagement metrics from providers into unified value
    pub fn resolve_unified_engagement(&self, provider_data: &[ProviderAnimeData]) -> Option<u32> {
        // Use the highest engagement value available (favorites, scored_by, etc.)
        provider_data.iter().filter_map(|d| d.favorites).max()
    }

    /// Merge anime data from multiple providers
    /// Creates unified anime with best available data from all sources
    pub fn merge_anime_data(
        &self,
        existing: &AnimeDetailed,
        new_provider_data: ProviderAnimeData,
    ) -> AnimeDetailed {
        let mut updated = existing.clone();

        // Collect all provider data including existing
        let mut all_provider_data = vec![
            ProviderAnimeData {
                provider: existing.provider_metadata.primary_provider.clone(),
                score: existing.score,
                favorites: existing.favorites,
                age_restriction: existing.age_restriction.clone(),
                last_synced: existing.last_synced_at,
            },
            new_provider_data,
        ];

        // Remove duplicates (keep newest data for same provider)
        all_provider_data.sort_by_key(|d| d.last_synced);
        all_provider_data.dedup_by_key(|d| d.provider.clone());

        // Resolve unified data
        updated.score = self.resolve_unified_score(&all_provider_data);
        updated.favorites = self.resolve_unified_engagement(&all_provider_data);
        updated.age_restriction = self.resolve_age_restriction(&all_provider_data);
        updated.updated_at = Utc::now();
        updated.last_synced_at = Some(Utc::now());

        updated
    }

    /// Normalize scores from different providers to 0-10 scale
    fn normalize_score_to_10_scale(&self, score: f32, provider: &AnimeProvider) -> f32 {
        match provider {
            AnimeProvider::AniList => {
                // AniList uses 0-100, convert to 0-10
                (score / 10.0).clamp(0.0, 10.0)
            }
            AnimeProvider::Jikan
            | AnimeProvider::Kitsu
            | AnimeProvider::AniDB
            | AnimeProvider::TMDB => {
                // These typically use 0-10 already
                score.clamp(0.0, 10.0)
            }
        }
    }

    /// Check if anime needs resyncing based on last sync time
    pub fn needs_resync(&self, anime: &AnimeDetailed, resync_threshold_hours: u64) -> bool {
        match anime.last_synced_at {
            None => true, // Never synced
            Some(last_sync) => {
                let threshold = chrono::Duration::hours(resync_threshold_hours as i64);
                Utc::now() - last_sync > threshold
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_age_rating_priority_resolution() {
        let resolver = UnifiedDataResolver::new();

        let provider_data = vec![
            ProviderAnimeData {
                provider: AnimeProvider::AniList,
                age_restriction: Some(UnifiedAgeRestriction::GeneralAudiences),
                score: None,
                favorites: None,
                last_synced: None,
            },
            ProviderAnimeData {
                provider: AnimeProvider::Jikan,
                age_restriction: Some(UnifiedAgeRestriction::ParentalGuidance13),
                score: None,
                favorites: None,
                last_synced: None,
            },
        ];

        // Should prefer Jikan (higher priority) over AniList
        let resolved = resolver.resolve_age_restriction(&provider_data);
        assert_eq!(resolved, Some(UnifiedAgeRestriction::ParentalGuidance13));
    }

    #[test]
    fn test_score_normalization() {
        let resolver = UnifiedDataResolver::new();

        // AniList score (0-100 scale)
        let anilist_normalized =
            resolver.normalize_score_to_10_scale(75.0, &AnimeProvider::AniList);
        assert_eq!(anilist_normalized, 7.5);

        // Jikan score (already 0-10 scale)
        let jikan_normalized = resolver.normalize_score_to_10_scale(8.5, &AnimeProvider::Jikan);
        assert_eq!(jikan_normalized, 8.5);
    }
}
