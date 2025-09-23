use crate::modules::anime::AnimeDetailed;
use crate::modules::provider::ProviderService;
use crate::shared::errors::AppResult;
use chrono::Datelike;
use std::collections::HashMap;
use std::sync::Arc;

use super::types::{DataQualityMetrics, EnhancedValidatedAnime};

/// Service for enhancing anime data quality during import process
/// Provides analysis, gap filling, and quality improvement suggestions
#[derive(Clone)]
pub struct DataEnhancementService {
    provider_service: Arc<ProviderService>,
}

/// Data enhancement result with improvement suggestions
#[derive(Debug, Clone)]
pub struct EnhancementResult {
    pub enhanced_anime: AnimeDetailed,
    pub improvements_made: Vec<String>,
    pub quality_score_before: f32,
    pub quality_score_after: f32,
    pub provider_sources: HashMap<String, String>, // field -> provider
}

/// Aggregate quality insights for a batch of anime
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct BatchQualityInsights {
    pub total_anime: u32,
    pub average_quality_before: f32,
    pub average_quality_after: f32,
    pub common_gaps: HashMap<String, u32>, // field -> count
    pub provider_effectiveness: HashMap<String, f32>, // provider -> avg quality
    pub enhancement_summary: Vec<String>,
}

impl DataEnhancementService {
    pub fn new(provider_service: Arc<ProviderService>) -> Self {
        Self { provider_service }
    }

    /// Enhance a single anime's data quality by filling gaps and improving completeness
    pub async fn enhance_anime_data(
        &self,
        anime: &AnimeDetailed,
        quality_metrics: &DataQualityMetrics,
    ) -> AppResult<EnhancementResult> {
        let mut enhanced_anime = anime.clone();
        let mut improvements_made = Vec::new();
        let mut provider_sources = HashMap::new();
        let quality_before = (quality_metrics.completeness_score
            + quality_metrics.consistency_score
            + quality_metrics.freshness_score
            + quality_metrics.source_reliability)
            / 4.0;

        // Identify fields that need enhancement based on quality metrics
        let gaps = self.identify_data_gaps(anime, quality_metrics);

        // Try to fill gaps using comprehensive provider data
        if !gaps.is_empty() {
            // Search for the same anime across all providers to get more complete data
            let search_query = &anime.title.main;
            match self
                .provider_service
                .search_anime_comprehensive(search_query, 3)
                .await
            {
                Ok(provider_results) => {
                    // Find the best match for this anime
                    if let Some(best_match) = self.find_best_match(anime, &provider_results) {
                        self.fill_data_gaps(
                            &mut enhanced_anime,
                            best_match,
                            &gaps,
                            &mut improvements_made,
                            &mut provider_sources,
                        );
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch comprehensive data for enhancement: {}", e);
                }
            }
        }

        // Calculate new quality score
        let enhanced_quality = self.calculate_enhanced_quality(&enhanced_anime);

        Ok(EnhancementResult {
            enhanced_anime,
            improvements_made,
            quality_score_before: quality_before,
            quality_score_after: (enhanced_quality.completeness_score
                + enhanced_quality.consistency_score
                + enhanced_quality.freshness_score
                + enhanced_quality.source_reliability)
                / 4.0,
            provider_sources,
        })
    }

    /// Enhance multiple anime and provide batch insights
    pub async fn enhance_batch(
        &self,
        enhanced_validated_anime: Vec<EnhancedValidatedAnime>,
    ) -> AppResult<(Vec<EnhancementResult>, BatchQualityInsights)> {
        let mut enhancement_results = Vec::new();
        let mut common_gaps: HashMap<String, u32> = HashMap::new();
        let mut provider_effectiveness: HashMap<String, Vec<f32>> = HashMap::new();

        let total_anime = enhanced_validated_anime.len() as u32;
        let mut total_quality_before = 0.0f32;
        let mut total_quality_after = 0.0f32;

        for enhanced_validated in enhanced_validated_anime {
            let anime = &enhanced_validated.anime_data;
            let quality_metrics = &enhanced_validated.data_quality;

            // Enhance individual anime
            match self.enhance_anime_data(anime, quality_metrics).await {
                Ok(enhancement_result) => {
                    total_quality_before += enhancement_result.quality_score_before;
                    total_quality_after += enhancement_result.quality_score_after;

                    // Track common gaps
                    for improvement in &enhancement_result.improvements_made {
                        *common_gaps.entry(improvement.clone()).or_insert(0) += 1;
                    }

                    // Track provider effectiveness
                    for (field, provider) in &enhancement_result.provider_sources {
                        provider_effectiveness
                            .entry(provider.clone())
                            .or_insert_with(Vec::new)
                            .push(enhancement_result.quality_score_after);
                    }

                    enhancement_results.push(enhancement_result);
                }
                Err(e) => {
                    log::warn!("Failed to enhance anime {}: {}", anime.title.main, e);
                    // Still count for averages but with original quality
                    let quality_before = (quality_metrics.completeness_score
                        + quality_metrics.consistency_score
                        + quality_metrics.freshness_score
                        + quality_metrics.source_reliability)
                        / 4.0;
                    total_quality_before += quality_before;
                    total_quality_after += quality_before;
                }
            }
        }

        // Calculate provider effectiveness averages
        let provider_avg_effectiveness: HashMap<String, f32> = provider_effectiveness
            .into_iter()
            .map(|(provider, scores)| {
                let avg = scores.iter().sum::<f32>() / scores.len() as f32;
                (provider, avg)
            })
            .collect();

        // Generate enhancement summary
        let mut enhancement_summary = Vec::new();
        if total_anime > 0 {
            let quality_improvement =
                (total_quality_after - total_quality_before) / total_anime as f32;
            enhancement_summary.push(format!(
                "Average quality improved by {:.2}% across {} anime",
                quality_improvement * 100.0,
                total_anime
            ));

            if let Some((most_common_gap, count)) =
                common_gaps.iter().max_by_key(|(_, &count)| count)
            {
                enhancement_summary.push(format!(
                    "Most common improvement: {} ({} anime affected)",
                    most_common_gap, count
                ));
            }

            if let Some((best_provider, avg_quality)) = provider_avg_effectiveness
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            {
                enhancement_summary.push(format!(
                    "Most effective provider for enhancements: {} (avg quality: {:.1}%)",
                    best_provider,
                    avg_quality * 100.0
                ));
            }
        }

        let insights = BatchQualityInsights {
            total_anime,
            average_quality_before: if total_anime > 0 {
                total_quality_before / total_anime as f32
            } else {
                0.0
            },
            average_quality_after: if total_anime > 0 {
                total_quality_after / total_anime as f32
            } else {
                0.0
            },
            common_gaps,
            provider_effectiveness: provider_avg_effectiveness,
            enhancement_summary,
        };

        Ok((enhancement_results, insights))
    }

    /// Identify data gaps that could be filled from other providers
    fn identify_data_gaps(
        &self,
        anime: &AnimeDetailed,
        quality_metrics: &DataQualityMetrics,
    ) -> Vec<String> {
        let mut gaps = Vec::new();

        // Check for missing or low-quality fields
        if anime.title.english.is_none()
            || anime.title.english.as_ref().map_or(true, |s| s.is_empty())
        {
            gaps.push("title.english".to_string());
        }
        if anime.title.japanese.is_none()
            || anime.title.japanese.as_ref().map_or(true, |s| s.is_empty())
        {
            gaps.push("title.japanese".to_string());
        }
        if anime.synopsis.is_none() || anime.synopsis.as_ref().map_or(true, |s| s.len() < 50) {
            gaps.push("synopsis".to_string());
        }
        if anime.genres.is_empty() {
            gaps.push("genres".to_string());
        }
        if anime.studios.is_empty() {
            gaps.push("studios".to_string());
        }
        if anime.score.is_none() {
            gaps.push("score".to_string());
        }
        if anime.aired.from.is_none() {
            gaps.push("aired_from".to_string());
        }
        if anime.image_url.is_none() {
            gaps.push("images".to_string());
        }

        // Only include gaps where quality metrics indicate improvement needed
        gaps.into_iter()
            .filter(|gap| {
                !quality_metrics
                    .field_completeness
                    .get(gap)
                    .unwrap_or(&false)
                    || quality_metrics.completeness_score < 0.8
            })
            .collect()
    }

    /// Find the best matching anime from provider results
    fn find_best_match<'a>(
        &self,
        target: &AnimeDetailed,
        candidates: &'a [AnimeDetailed],
    ) -> Option<&'a AnimeDetailed> {
        candidates
            .iter()
            .max_by(|a, b| {
                let score_a = self.calculate_match_score(target, a);
                let score_b = self.calculate_match_score(target, b);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .filter(|candidate| self.calculate_match_score(target, candidate) > 0.7)
        // Only use if good match
    }

    /// Calculate how well a candidate matches the target anime
    fn calculate_match_score(&self, target: &AnimeDetailed, candidate: &AnimeDetailed) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Title similarity (most important)
        if self.titles_similar(&target.title.main, &candidate.title.main) {
            score += 0.4;
        }
        factors += 1;

        // Year similarity (if both have aired dates)
        if let (Some(target_date), Some(candidate_date)) =
            (&target.aired.from, &candidate.aired.from)
        {
            if target_date.year() == candidate_date.year() {
                score += 0.3;
            }
            factors += 1;
        }

        // Episode count similarity (if both have episode counts)
        let target_eps = target.episodes.unwrap_or(0);
        let candidate_eps = candidate.episodes.unwrap_or(0);
        if target_eps > 0 && candidate_eps > 0 {
            let episode_diff = (target_eps as i32 - candidate_eps as i32).abs();
            if episode_diff <= 2 {
                score += 0.2;
            }
            factors += 1;
        }

        // Genre overlap
        if !target.genres.is_empty() && !candidate.genres.is_empty() {
            let target_genres: std::collections::HashSet<_> = target.genres.iter().collect();
            let candidate_genres: std::collections::HashSet<_> = candidate.genres.iter().collect();
            let overlap = target_genres.intersection(&candidate_genres).count();
            let overlap_ratio =
                overlap as f64 / target_genres.len().max(candidate_genres.len()) as f64;
            score += 0.1 * overlap_ratio;
            factors += 1;
        }

        if factors > 0 {
            score
        } else {
            0.0
        }
    }

    /// Check if two titles are similar (simple similarity check)
    fn titles_similar(&self, title1: &str, title2: &str) -> bool {
        let title1_clean = title1.to_lowercase().trim().to_string();
        let title2_clean = title2.to_lowercase().trim().to_string();

        // Exact match
        if title1_clean == title2_clean {
            return true;
        }

        // One contains the other
        if title1_clean.contains(&title2_clean) || title2_clean.contains(&title1_clean) {
            return true;
        }

        // Simple word-based similarity
        let words1: std::collections::HashSet<&str> = title1_clean.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = title2_clean.split_whitespace().collect();
        let common_words = words1.intersection(&words2).count();
        let total_words = words1.len().max(words2.len());

        if total_words > 0 {
            let similarity = common_words as f64 / total_words as f64;
            similarity > 0.6
        } else {
            false
        }
    }

    /// Fill data gaps in the target anime using data from the source
    fn fill_data_gaps(
        &self,
        target: &mut AnimeDetailed,
        source: &AnimeDetailed,
        gaps: &[String],
        improvements_made: &mut Vec<String>,
        provider_sources: &mut HashMap<String, String>,
    ) {
        for gap in gaps {
            match gap.as_str() {
                "title.english" => {
                    if let Some(ref english_title) = source.title.english {
                        if !english_title.is_empty() && target.title.english.is_none() {
                            target.title.english = Some(english_title.clone());
                            improvements_made.push("Added English title".to_string());
                            let external_id = source
                                .provider_metadata
                                .external_ids
                                .get(&source.provider_metadata.primary_provider)
                                .cloned()
                                .unwrap_or_default();
                            provider_sources.insert("title.english".to_string(), external_id);
                        }
                    }
                }
                "title.japanese" => {
                    if let Some(ref japanese_title) = source.title.japanese {
                        if !japanese_title.is_empty() && target.title.japanese.is_none() {
                            target.title.japanese = Some(japanese_title.clone());
                            improvements_made.push("Added Japanese title".to_string());
                            let external_id = source
                                .provider_metadata
                                .external_ids
                                .get(&source.provider_metadata.primary_provider)
                                .cloned()
                                .unwrap_or_default();
                            provider_sources.insert("title.japanese".to_string(), external_id);
                        }
                    }
                }
                "synopsis" => {
                    if let Some(ref synopsis) = source.synopsis {
                        if synopsis.len() > 50
                            && (target.synopsis.is_none()
                                || target
                                    .synopsis
                                    .as_ref()
                                    .map_or(true, |s| s.len() < synopsis.len()))
                        {
                            target.synopsis = Some(synopsis.clone());
                            improvements_made.push("Enhanced synopsis".to_string());
                            let external_id = source
                                .provider_metadata
                                .external_ids
                                .get(&source.provider_metadata.primary_provider)
                                .cloned()
                                .unwrap_or_default();
                            provider_sources.insert("synopsis".to_string(), external_id);
                        }
                    }
                }
                "genres" => {
                    if !source.genres.is_empty() && target.genres.len() < source.genres.len() {
                        // Merge genres, avoiding duplicates
                        for genre in &source.genres {
                            if !target.genres.contains(genre) {
                                target.genres.push(genre.clone());
                            }
                        }
                        improvements_made.push(format!(
                            "Added {} genres",
                            source.genres.len() - target.genres.len()
                        ));
                        let external_id = source
                            .provider_metadata
                            .external_ids
                            .get(&source.provider_metadata.primary_provider)
                            .cloned()
                            .unwrap_or_default();
                        provider_sources.insert("genres".to_string(), external_id);
                    }
                }
                "studios" => {
                    if !source.studios.is_empty() && target.studios.is_empty() {
                        target.studios = source.studios.clone();
                        improvements_made.push("Added studio information".to_string());
                        let external_id = source
                            .provider_metadata
                            .external_ids
                            .get(&source.provider_metadata.primary_provider)
                            .cloned()
                            .unwrap_or_default();
                        provider_sources.insert("studios".to_string(), external_id);
                    }
                }
                "score" => {
                    if source.score.is_some() && target.score.is_none() {
                        target.score = source.score;
                        improvements_made.push("Added rating score".to_string());
                        let external_id = source
                            .provider_metadata
                            .external_ids
                            .get(&source.provider_metadata.primary_provider)
                            .cloned()
                            .unwrap_or_default();
                        provider_sources.insert("score".to_string(), external_id);
                    }
                }
                "aired_from" => {
                    if source.aired.from.is_some() && target.aired.from.is_none() {
                        target.aired.from = source.aired.from;
                        improvements_made.push("Added air date".to_string());
                        let external_id = source
                            .provider_metadata
                            .external_ids
                            .get(&source.provider_metadata.primary_provider)
                            .cloned()
                            .unwrap_or_default();
                        provider_sources.insert("aired_from".to_string(), external_id);
                    }
                }
                "images" => {
                    if source.image_url.is_some() && target.image_url.is_none() {
                        target.image_url = source.image_url.clone();
                        improvements_made.push("Added cover images".to_string());
                        let external_id = source
                            .provider_metadata
                            .external_ids
                            .get(&source.provider_metadata.primary_provider)
                            .cloned()
                            .unwrap_or_default();
                        provider_sources.insert("images".to_string(), external_id);
                    }
                }
                _ => {}
            }
        }
    }

    /// Calculate enhanced quality metrics for the improved anime
    fn calculate_enhanced_quality(&self, anime: &AnimeDetailed) -> DataQualityMetrics {
        let completeness_score = self.calculate_completeness_score(anime);
        let consistency_score = self.calculate_consistency_score(anime);
        let field_completeness = self.calculate_field_completeness(anime);

        DataQualityMetrics {
            completeness_score,
            consistency_score,
            freshness_score: 1.0,    // Assume fresh since we just enhanced it
            source_reliability: 0.8, // Good reliability after enhancement
            field_completeness,
            provider_agreements: HashMap::new(), // Would be filled during aggregation
        }
    }

    fn calculate_completeness_score(&self, anime: &AnimeDetailed) -> f32 {
        let mut filled_fields = 0;
        let total_fields = 10; // Total important fields we track

        // Core fields
        if !anime.title.main.is_empty() {
            filled_fields += 1;
        }
        if anime.title.english.is_some() {
            filled_fields += 1;
        }
        if anime.title.japanese.is_some() {
            filled_fields += 1;
        }
        if anime.synopsis.is_some() {
            filled_fields += 1;
        }
        if !anime.genres.is_empty() {
            filled_fields += 1;
        }
        if !anime.studios.is_empty() {
            filled_fields += 1;
        }
        if anime.score.is_some() {
            filled_fields += 1;
        }
        if anime.aired.from.is_some() {
            filled_fields += 1;
        }
        if anime.image_url.is_some() {
            filled_fields += 1;
        }
        if anime.episodes.is_some() {
            filled_fields += 1;
        }

        filled_fields as f32 / total_fields as f32
    }

    fn calculate_consistency_score(&self, anime: &AnimeDetailed) -> f32 {
        let mut score: f32 = 1.0;

        // Check for basic consistency issues
        if let Some(episodes) = anime.episodes {
            if episodes == 0 {
                score -= 0.1;
            }
        }

        if let Some(score_val) = anime.score {
            if score_val < 0.0 || score_val > 10.0 {
                score -= 0.2;
            }
        }

        // Check title consistency
        if let Some(ref english) = anime.title.english {
            if english.is_empty() {
                score -= 0.1;
            }
        }

        score.max(0.0)
    }

    fn calculate_field_completeness(&self, anime: &AnimeDetailed) -> HashMap<String, bool> {
        let mut completeness = HashMap::new();

        completeness.insert("title.main".to_string(), !anime.title.main.is_empty());
        completeness.insert("title.english".to_string(), anime.title.english.is_some());
        completeness.insert("title.japanese".to_string(), anime.title.japanese.is_some());
        completeness.insert("synopsis".to_string(), anime.synopsis.is_some());
        completeness.insert("genres".to_string(), !anime.genres.is_empty());
        completeness.insert("studios".to_string(), !anime.studios.is_empty());
        completeness.insert("score".to_string(), anime.score.is_some());
        completeness.insert("aired_from".to_string(), anime.aired.from.is_some());
        completeness.insert("images".to_string(), anime.image_url.is_some());
        completeness.insert("episodes".to_string(), anime.episodes.is_some());

        completeness
    }
}
