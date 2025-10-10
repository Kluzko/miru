use std::collections::HashMap;

use crate::{
    modules::{
        anime::{domain::services::score_calculator::ScoreCalculator, AnimeDetailed},
        provider::AnimeProvider,
    },
    shared::errors::{AppError, AppResult},
};

use crate::modules::provider::domain::entities::anime_data::{AnimeData, DataQuality};

/// Service for assessing and improving data quality
#[derive(Clone)]
pub struct DataQualityService {
    score_calculator: ScoreCalculator,
}

impl DataQualityService {
    pub fn new() -> Self {
        Self {
            score_calculator: ScoreCalculator::new(),
        }
    }

    /// Assess the quality of anime data
    pub fn assess_quality(&self, anime: &AnimeDetailed) -> DataQuality {
        DataQuality::calculate(anime)
    }

    /// Merge anime data from multiple providers to improve quality
    pub fn merge_anime_data(&self, anime_data_list: Vec<AnimeData>) -> AppResult<AnimeData> {
        if anime_data_list.is_empty() {
            return Err(AppError::InvalidInput("No anime data to merge".to_string()));
        }

        if anime_data_list.len() == 1 {
            return Ok(anime_data_list.into_iter().next().unwrap());
        }

        // Use the highest quality data as base
        let mut sorted_data = anime_data_list;
        sorted_data.sort_by(|a, b| {
            b.quality
                .score
                .partial_cmp(&a.quality.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut base_anime = sorted_data[0].clone();
        let other_data = &sorted_data[1..];

        // Merge data from other sources to fill gaps
        base_anime = self.fill_missing_fields(base_anime, other_data);

        // Recalculate quality after merging
        base_anime.quality = DataQuality::calculate(&base_anime.anime);

        // Merge quality metrics
        let all_qualities: Vec<DataQuality> =
            sorted_data.iter().map(|d| d.quality.clone()).collect();
        base_anime.quality = DataQuality::merge(&all_qualities);

        // Update source information
        let all_providers: Vec<AnimeProvider> = sorted_data
            .iter()
            .map(|d| d.source.primary_provider)
            .collect();
        base_anime.source.providers_used = all_providers;

        // Update confidence based on multiple sources
        base_anime.source.confidence = self.calculate_merged_confidence(&sorted_data);

        Ok(base_anime)
    }

    /// Intelligently merge and assess data from multiple providers
    fn fill_missing_fields(&self, mut base: AnimeData, other_data: &[AnimeData]) -> AnimeData {
        for other in other_data {
            // UNIFIED SCORE ASSESSMENT - Weighted average based on voter/user count
            base.anime.score = self.calculate_composite_score(&base, other);
            base.anime.rating = base.anime.score; // Keep rating in sync with score

            // INTELLIGENT TEXT MERGING
            base.anime.description =
                self.select_best_description(&base.anime.description, &other.anime.description);

            // TITLE SYNONYMS - Merge and deduplicate
            self.merge_synonyms(&mut base.anime.title.synonyms, &other.anime.title.synonyms);

            // PROVIDER-STRENGTH-BASED FIELD SELECTION

            // Age restrictions: Prefer MAL (Jikan) data as it's more detailed
            if base.anime.age_restriction.is_none() && other.anime.age_restriction.is_some() {
                log::info!(
                    "MERGE: Using age_restriction from {:?}",
                    other.source.primary_provider
                );
                base.anime.age_restriction = other.anime.age_restriction.clone();
            }

            // Images: Prefer AniList for better quality, fallback to MAL
            base.anime.image_url = self.select_best_image(
                &base.anime.image_url,
                &other.anime.image_url,
                &other.source.primary_provider,
            );
            base.anime.images = base.anime.image_url.clone();

            // Banner: Only AniList provides this
            if base.anime.banner_image.is_none() && other.anime.banner_image.is_some() {
                base.anime.banner_image = other.anime.banner_image.clone();
            }

            // Basic field filling (same as before)
            if base.anime.episodes.is_none() && other.anime.episodes.is_some() {
                base.anime.episodes = other.anime.episodes;
            }

            // Merge genres (deduplicate)
            for genre in &other.anime.genres {
                if !base.anime.genres.iter().any(|g| g.name == genre.name) {
                    base.anime.genres.push(genre.clone());
                }
            }

            // Air dates
            if base.anime.aired.from.is_none() && other.anime.aired.from.is_some() {
                base.anime.aired.from = other.anime.aired.from;
            }
            if base.anime.aired.to.is_none() && other.anime.aired.to.is_some() {
                base.anime.aired.to = other.anime.aired.to;
            }

            // Status
            if base.anime.status.is_empty() && !other.anime.status.is_empty() {
                base.anime.status = other.anime.status.clone();
            }
        }

        // UPDATE UNIFIED ASSESSMENTS using anime module calculators
        base.anime.composite_score = self.score_calculator.calculate_composite_score(&base.anime);
        base.anime.tier = self
            .score_calculator
            .determine_tier(base.anime.composite_score);
        base.anime.quality_metrics = self.score_calculator.calculate_quality_metrics(&base.anime);

        base
    }

    /// Calculate composite score from multiple providers using weighted average
    fn calculate_composite_score(&self, base: &AnimeData, other: &AnimeData) -> Option<f32> {
        match (
            base.anime.score,
            other.anime.score,
            base.anime.favorites,
            other.anime.favorites,
        ) {
            (Some(base_score), Some(other_score), base_fav, other_fav) => {
                // Use favorites as weight (more favorites = more reliable score)
                let base_weight = base_fav.unwrap_or(100) as f32;
                let other_weight = other_fav.unwrap_or(100) as f32;

                let weighted_score = (base_score * base_weight + other_score * other_weight)
                    / (base_weight + other_weight);
                // Round to 2 decimal places for consistency
                Some((weighted_score * 100.0).round() / 100.0)
            }
            (Some(score), None, _, _) | (None, Some(score), _, _) => Some(score),
            (None, None, _, _) => None,
        }
    }
    

    /// Select best description (prefer longer, more detailed)
    fn select_best_description(
        &self,
        base: &Option<String>,
        other: &Option<String>,
    ) -> Option<String> {
        match (base, other) {
            (Some(base_desc), Some(other_desc)) => {
                if other_desc.len() > base_desc.len() {
                    Some(other_desc.clone())
                } else {
                    Some(base_desc.clone())
                }
            }
            (Some(desc), None) | (None, Some(desc)) => Some(desc.clone()),
            (None, None) => None,
        }
    }

    /// Merge synonyms and deduplicate
    fn merge_synonyms(&self, base_synonyms: &mut Vec<String>, other_synonyms: &[String]) {
        for synonym in other_synonyms {
            if !base_synonyms.contains(synonym) {
                base_synonyms.push(synonym.clone());
            }
        }
    }

    /// Select best image based on provider strength
    fn select_best_image(
        &self,
        base: &Option<String>,
        other: &Option<String>,
        other_provider: &crate::modules::provider::AnimeProvider,
    ) -> Option<String> {
        match (base, other) {
            // Prefer AniList images (higher quality)
            (_, Some(other_url))
                if *other_provider == crate::modules::provider::AnimeProvider::AniList =>
            {
                Some(other_url.clone())
            }
            // Fallback to any available image
            (Some(base_url), None) => Some(base_url.clone()),
            (None, Some(other_url)) => Some(other_url.clone()),
            (Some(base_url), Some(_)) => Some(base_url.clone()), // Keep base if both exist
            (None, None) => None,
        }
    }

    /// Calculate tier based on unified assessments
    fn calculate_tier(
        &self,
        anime_data: &AnimeData,
    ) -> crate::modules::anime::domain::value_objects::AnimeTier {
        let score = anime_data.anime.score.unwrap_or(0.0);
        let favorites = anime_data.anime.favorites.unwrap_or(0) as f32;

        // Tier calculation based on score and popularity
        match score {
            s if s >= 9.0 && favorites > 1000.0 => {
                crate::modules::anime::domain::value_objects::AnimeTier::S
            }
            s if s >= 8.5 => crate::modules::anime::domain::value_objects::AnimeTier::A,
            s if s >= 7.5 => crate::modules::anime::domain::value_objects::AnimeTier::B,
            s if s >= 6.0 => crate::modules::anime::domain::value_objects::AnimeTier::C,
            _ => crate::modules::anime::domain::value_objects::AnimeTier::D,
        }
    }

    /// Calculate real quality metrics from actual data
    fn calculate_real_quality_metrics(
        &self,
        base: &AnimeData,
        other_data: &[AnimeData],
    ) -> crate::modules::anime::domain::value_objects::QualityMetrics {
        let total_favorites: u32 = std::iter::once(base)
            .chain(other_data.iter())
            .filter_map(|data| data.anime.favorites)
            .sum();

        let avg_score = std::iter::once(base)
            .chain(other_data.iter())
            .filter_map(|data| data.anime.score)
            .fold((0.0, 0), |(sum, count), score| (sum + score, count + 1));

        crate::modules::anime::domain::value_objects::QualityMetrics {
            popularity_score: (total_favorites as f32 / 10000.0).min(1.0), // Normalize to 0-1
            engagement_score: if avg_score.1 > 0 {
                avg_score.0 / avg_score.1 as f32 / 10.0
            } else {
                0.0
            },
            consistency_score: if other_data.is_empty() { 0.5 } else { 0.9 }, // High if multiple sources agree
            audience_reach_score: (total_favorites as f32).ln().max(0.0) / 10.0, // Log scale for reach
        }
    }

    /// Calculate confidence score when merging multiple sources
    fn calculate_merged_confidence(&self, data_list: &[AnimeData]) -> f32 {
        if data_list.len() == 1 {
            return data_list[0].source.confidence;
        }

        // Higher confidence when multiple high-quality sources agree
        let avg_quality =
            data_list.iter().map(|d| d.quality.score).sum::<f32>() / data_list.len() as f32;
        let multi_source_bonus = (data_list.len() as f32).min(3.0) * 0.1; // Max 30% bonus

        (avg_quality + multi_source_bonus).min(1.0)
    }

    /// Check if anime data meets minimum quality standards
    pub fn meets_standards(&self, anime_data: &AnimeData, min_score: f32) -> bool {
        anime_data.quality.score >= min_score && anime_data.meets_quality_threshold()
    }

    /// Get quality improvement suggestions
    pub fn get_improvement_suggestions(&self, anime_data: &AnimeData) -> Vec<String> {
        let mut suggestions = Vec::new();

        if anime_data.quality.completeness < 0.8 {
            suggestions.push("Try additional providers to fill missing data".to_string());
        }

        if anime_data.quality.consistency < 0.7 {
            suggestions.push("Data inconsistencies detected across providers".to_string());
        }

        if anime_data.quality.missing_fields.len() > 3 {
            suggestions.push(format!(
                "Many fields missing: {}",
                anime_data.quality.missing_fields.join(", ")
            ));
        }

        if anime_data.source.providers_used.len() == 1 {
            suggestions
                .push("Consider using multiple providers for better data quality".to_string());
        }

        suggestions
    }

    /// Rank providers by data quality for a given anime type
    pub fn rank_providers_by_quality(
        &self,
        provider_data: HashMap<AnimeProvider, Vec<AnimeData>>,
    ) -> Vec<(AnimeProvider, f32)> {
        let mut provider_scores = Vec::new();

        for (provider, data_list) in provider_data {
            if data_list.is_empty() {
                provider_scores.push((provider, 0.0));
                continue;
            }

            let avg_score =
                data_list.iter().map(|d| d.quality.score).sum::<f32>() / data_list.len() as f32;
            provider_scores.push((provider, avg_score));
        }

        // Sort by score descending
        provider_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        provider_scores
    }
}

impl Default for DataQualityService {
    fn default() -> Self {
        Self::new()
    }
}
