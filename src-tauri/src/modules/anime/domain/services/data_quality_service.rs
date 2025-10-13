use crate::modules::provider::domain::entities::anime_data::{AnimeData, DataQuality};
use crate::{
    modules::{anime::AnimeDetailed, provider::AnimeProvider},
    shared::errors::{AppError, AppResult},
};
use std::collections::HashMap;

// Import the new merging architecture
use super::{
    data_merging::{DefaultMergeStrategy, MergeContext, MergeStrategy},
    score_calculator::ScoreCalculator,
};

/// Service for assessing and improving data quality
///
/// Responsibilities (following Single Responsibility Principle):
/// - Quality assessment
/// - Data merging coordination (delegates to merge strategies)
/// - Quality metrics calculation
/// - Provider ranking
#[derive(Clone)]
pub struct DataQualityService {
    score_calculator: ScoreCalculator,
    merge_strategy: DefaultMergeStrategy,
}

impl DataQualityService {
    pub fn new() -> Self {
        Self {
            score_calculator: ScoreCalculator::new(),
            merge_strategy: DefaultMergeStrategy::new(),
        }
    }

    /// Assess the quality of anime data
    pub fn assess_quality(&self, anime: &AnimeDetailed) -> DataQuality {
        DataQuality::calculate(anime)
    }

    /// Merge anime data from multiple providers to improve quality
    ///
    /// This is the main entry point for data merging. It:
    /// 1. Validates input
    /// 2. Sorts by quality (highest first)
    /// 3. Creates merge context
    /// 4. Delegates to merge strategy
    /// 5. Updates final metadata
    pub fn merge_anime_data(&self, anime_data_list: Vec<AnimeData>) -> AppResult<AnimeData> {
        // Validation
        if anime_data_list.is_empty() {
            return Err(AppError::InvalidInput("No anime data to merge".to_string()));
        }

        if anime_data_list.len() == 1 {
            return Ok(anime_data_list.into_iter().next().unwrap());
        }

        // Sort by quality score (highest quality becomes base)
        let mut sorted_data = anime_data_list;
        sorted_data.sort_by(|a, b| {
            b.quality
                .score
                .partial_cmp(&a.quality.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        log::info!(
            "MERGE: Starting merge of {} data sources. Base provider: {:?} (quality: {:.2})",
            sorted_data.len(),
            sorted_data[0].source.primary_provider,
            sorted_data[0].quality.score
        );

        // Create merge context with base and other sources
        let base = sorted_data[0].clone();
        let sources = sorted_data[1..].to_vec();
        let context = MergeContext::new(base, sources);

        // Use strategy to merge
        let merged = self.merge_strategy.merge(context)?;

        // Update final quality calculations
        self.finalize_quality_metrics(merged)
    }

    /// Finalize quality metrics after merging
    fn finalize_quality_metrics(&self, mut merged: AnimeData) -> AppResult<AnimeData> {
        // Recalculate quality based on final merged data
        merged.quality = DataQuality::calculate(&merged.anime);

        // Update tier and quality metrics using score calculator
        merged.anime.composite_score = self
            .score_calculator
            .calculate_composite_score(&merged.anime);
        merged.anime.tier = self
            .score_calculator
            .determine_tier(merged.anime.composite_score);
        merged.anime.quality_metrics = self
            .score_calculator
            .calculate_quality_metrics(&merged.anime);

        log::info!(
            "MERGE: Completed. Final quality: {:.2}, completeness: {:.2}, providers: {:?}",
            merged.quality.score,
            merged.quality.completeness,
            merged.source.providers_used
        );

        Ok(merged)
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

    /// Rank providers by data quality
    ///
    /// This helps identify which providers give the best data for your use case
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

    /// Public wrapper to calculate composite score for an anime
    pub fn calculate_anime_composite_score(&self, anime: &AnimeDetailed) -> f32 {
        self.score_calculator.calculate_composite_score(anime)
    }

    /// Public wrapper to calculate quality metrics for an anime
    pub fn calculate_anime_quality_metrics(
        &self,
        anime: &AnimeDetailed,
    ) -> crate::modules::anime::domain::value_objects::QualityMetrics {
        self.score_calculator.calculate_quality_metrics(anime)
    }

    /// Public wrapper to determine tier based on score
    pub fn determine_anime_tier(
        &self,
        score: f32,
    ) -> crate::modules::anime::domain::value_objects::AnimeTier {
        self.score_calculator.determine_tier(score)
    }
}

impl Default for DataQualityService {
    fn default() -> Self {
        Self::new()
    }
}
