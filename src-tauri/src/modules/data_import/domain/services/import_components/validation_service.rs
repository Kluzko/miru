use crate::modules::anime::AnimeRepository;
use crate::modules::provider::application::service::ProviderService;
use crate::shared::errors::{AppError, AppResult};
use crate::shared::utils::logger::{LogContext, TimedOperation};
use crate::{log_info, log_warn};
use chrono::Datelike;
use tauri::{AppHandle, Emitter};

use std::sync::Arc;

use super::types::{
    DataQualityMetrics, DataQualitySummary, EnhancedValidatedAnime, EnhancedValidationResult,
    ExistingAnime, ImportError, ValidatedAnime,
};

/// Progress event structure for real-time validation updates
#[derive(Clone, serde::Serialize)]
struct ValidationProgress {
    current: u32,
    total: u32,
    percentage: f32,
    current_title: String,
    status: String,
    found_count: u32,
    existing_count: u32,
    failed_count: u32,
    average_confidence: f32,
    providers_used: u32,
}

/// Detailed progress information for better UX
#[derive(Clone, serde::Serialize)]
struct ValidationStepInfo {
    step: String,
    description: String,
    title: String,
    provider: String,
    estimated_time_remaining: u32, // seconds
}

/// Handles validation logic using existing patterns
#[derive(Clone)]
pub struct ValidationService {
    anime_repo: Arc<dyn AnimeRepository>,
    provider_service: Arc<ProviderService>,
}

impl ValidationService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        provider_service: Arc<ProviderService>,
    ) -> Self {
        Self {
            anime_repo,
            provider_service,
        }
    }

    /// Helper method to get primary external ID and provider from anime (reused from existing)
    pub fn get_primary_external_info(
        anime: &crate::modules::anime::AnimeDetailed,
    ) -> (String, crate::modules::provider::AnimeProvider) {
        let provider = anime.provider_metadata.primary_provider.clone();
        let external_id = anime
            .provider_metadata
            .get_external_id(&provider)
            .cloned()
            .unwrap_or_else(|| "0".to_string());
        (external_id, provider)
    }

    /// Helper method to check if external ID is valid (reused from existing)
    pub fn is_valid_external_id(external_id: &str) -> bool {
        crate::shared::utils::Validator::is_valid_external_id(external_id)
    }

    /// Search for anime using comprehensive provider aggregation (enhanced with multi-provider data)
    pub async fn search_anime_multi_provider(
        &self,
        query: &str,
    ) -> AppResult<Vec<crate::modules::anime::AnimeDetailed>> {
        match self.provider_service.search_anime(query, 1).await {
            Ok(results) if !results.is_empty() => {
                LogContext::search_operation(query, Some("provider_service"), Some(results.len()));
                Ok(results)
            }
            Ok(_) => {
                LogContext::search_operation(query, Some("provider_service"), Some(0));
                Ok(vec![])
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    &format!("Provider search failed for '{}'", query),
                );
                Err(e)
            }
        }
    }

    /// Validate a single anime title using existing DB-first logic
    pub async fn validate_single_title(&self, title: &str) -> ValidationSingleResult {
        let item_timer = TimedOperation::new("validate_single_title");

        // STEP 1: Check database first using title variations (reused from existing)
        match self.anime_repo.find_by_title_variations(title).await {
            Ok(Some(existing_anime)) => {
                item_timer.finish();
                return ValidationSingleResult::AlreadyExists(ExistingAnime {
                    input_title: title.to_string(),
                    matched_title: existing_anime.title.main.clone(),
                    matched_field: "title_match".to_string(),
                    anime: existing_anime,
                });
            }
            Ok(None) => {
                // Continue to provider lookup
            }
            Err(e) => {
                log_warn!("Database lookup failed for '{}': {}", title, e);
                // Continue to provider lookup as fallback
            }
        }

        // STEP 2: Search providers only if not found in DB (reused from existing)
        match self.search_anime_multi_provider(title).await {
            Ok(anime_list) if !anime_list.is_empty() => {
                let anime = anime_list.into_iter().next().unwrap();
                let (external_id, provider) = Self::get_primary_external_info(&anime);

                if Self::is_valid_external_id(&external_id) {
                    // STEP 3: Double-check by external_id to avoid duplicates (reused from existing)
                    match self
                        .anime_repo
                        .find_by_external_id(&provider, &external_id)
                        .await
                    {
                        Ok(Some(existing)) => {
                            item_timer.finish();
                            ValidationSingleResult::AlreadyExists(ExistingAnime {
                                input_title: title.to_string(),
                                matched_title: existing.title.main.clone(),
                                matched_field: format!("{:?}_id", provider),
                                anime: existing,
                            })
                        }
                        Ok(None) => {
                            item_timer.finish();
                            ValidationSingleResult::Found(ValidatedAnime {
                                input_title: title.to_string(),
                                anime_data: anime,
                            })
                        }
                        Err(e) => {
                            let context_error = AppError::DatabaseError(format!(
                                "External ID lookup failed for '{}' with provider {:?}: {}",
                                title, provider, e
                            ));
                            LogContext::error_with_context(
                                &context_error,
                                "External ID validation failed",
                            );
                            item_timer.finish();
                            ValidationSingleResult::Failed(ImportError {
                                title: title.to_string(),
                                reason: format!("Database error during external ID check: {}", e),
                            })
                        }
                    }
                } else {
                    log_warn!("Invalid external ID for '{}': {}", title, external_id);
                    item_timer.finish();
                    ValidationSingleResult::Found(ValidatedAnime {
                        input_title: title.to_string(),
                        anime_data: anime,
                    })
                }
            }
            Ok(_) => {
                item_timer.finish();
                ValidationSingleResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: "No results found on any provider".to_string(),
                })
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    &format!("Provider search failed for '{}'", title),
                );
                item_timer.finish();
                ValidationSingleResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: format!("Provider search failed: {}", e),
                })
            }
        }
    }

    // ========================================================================
    // ENHANCED VALIDATION METHODS WITH COMPREHENSIVE DATA ANALYSIS
    // ========================================================================

    /// Enhanced validation that uses comprehensive provider data and quality assessment
    pub async fn validate_single_title_enhanced(
        &self,
        title: &str,
    ) -> EnhancedValidationSingleResult {
        let item_timer = TimedOperation::new("validate_single_title_enhanced");

        // STEP 1: Check database first - return existing anime directly (no double lookup)
        match self.anime_repo.find_by_title_variations(title).await {
            Ok(Some(existing_anime)) => {
                item_timer.finish();
                return EnhancedValidationSingleResult::AlreadyExists(ExistingAnime {
                    input_title: title.to_string(),
                    matched_title: existing_anime.title.main.clone(),
                    matched_field: "database_title_match".to_string(),
                    anime: existing_anime,
                });
            }
            Ok(None) => {
                // Continue to comprehensive provider search
            }
            Err(e) => {
                log_warn!("Database lookup failed for '{}': {}", title, e);
                // Continue to provider lookup as fallback
            }
        }

        // STEP 2: Use comprehensive search for enhanced data
        match self.search_anime_multi_provider(title).await {
            Ok(anime_list) if !anime_list.is_empty() => {
                let anime = anime_list.into_iter().next().unwrap();
                let (external_id, provider) = Self::get_primary_external_info(&anime);

                // STEP 3: Re-check external ID to avoid duplicates (critical fix)
                if Self::is_valid_external_id(&external_id) {
                    match self
                        .anime_repo
                        .find_by_external_id(&provider, &external_id)
                        .await
                    {
                        Ok(Some(existing)) => {
                            item_timer.finish();
                            return EnhancedValidationSingleResult::AlreadyExists(ExistingAnime {
                                input_title: title.to_string(),
                                matched_title: existing.title.main.clone(),
                                matched_field: format!("{:?}_id", provider),
                                anime: existing,
                            });
                        }
                        Ok(None) => {
                            // Continue to quality analysis
                        }
                        Err(e) => {
                            log_warn!("External ID lookup failed for '{}': {}", title, e);
                            // Continue to quality analysis as fallback
                        }
                    }
                }

                // STEP 4: Analyze data quality and generate comprehensive metrics
                let data_quality = self.analyze_anime_data_quality(&anime).await;
                let confidence_score = self.calculate_confidence_score(&anime, &data_quality);
                let provider_sources = self.extract_provider_sources(&anime);

                item_timer.finish();
                EnhancedValidationSingleResult::Found(EnhancedValidatedAnime {
                    input_title: title.to_string(),
                    anime_data: anime,
                    data_quality,
                    provider_sources,
                    confidence_score,
                })
            }
            Ok(_) => {
                item_timer.finish();
                EnhancedValidationSingleResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: "No results found on any provider".to_string(),
                })
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    &format!("Enhanced provider search failed for '{}'", title),
                );
                item_timer.finish();
                EnhancedValidationSingleResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: format!("Provider search failed: {}", e),
                })
            }
        }
    }

    /// Analyze the quality of anime data from comprehensive provider aggregation
    async fn analyze_anime_data_quality(
        &self,
        anime: &crate::modules::anime::AnimeDetailed,
    ) -> DataQualityMetrics {
        let mut field_completeness = std::collections::HashMap::new();
        let mut total_fields = 0;
        let mut complete_fields = 0;

        // Analyze core fields completeness
        self.check_field_completeness(
            "title.main",
            !anime.title.main.is_empty(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );
        self.check_field_completeness(
            "title.english",
            anime.title.english.is_some(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );
        self.check_field_completeness(
            "title.japanese",
            anime.title.japanese.is_some(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );
        self.check_field_completeness(
            "synopsis",
            anime.synopsis.is_some(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );
        self.check_field_completeness(
            "score",
            anime.score.is_some(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );
        self.check_field_completeness(
            "episodes",
            anime.episodes.is_some(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );
        self.check_field_completeness(
            "image_url",
            anime.image_url.is_some(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );
        self.check_field_completeness(
            "genres",
            !anime.genres.is_empty(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );
        self.check_field_completeness(
            "studios",
            !anime.studios.is_empty(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );
        self.check_field_completeness(
            "aired_dates",
            anime.aired.from.is_some(),
            &mut field_completeness,
            &mut total_fields,
            &mut complete_fields,
        );

        let completeness_score = if total_fields > 0 {
            complete_fields as f32 / total_fields as f32
        } else {
            0.0
        };

        // Analyze provider reliability based on data sources
        let source_reliability =
            self.calculate_source_reliability(&anime.provider_metadata.primary_provider);

        // Calculate realistic consistency and freshness scores
        let consistency_score = self.calculate_consistency_score(&anime);
        let freshness_score = self.calculate_freshness_score(&anime);

        // Provider agreements - simplified for now
        let mut provider_agreements = std::collections::HashMap::new();
        provider_agreements.insert("primary_data".to_string(), 1);

        DataQualityMetrics {
            completeness_score,
            consistency_score,
            freshness_score,
            source_reliability,
            field_completeness,
            provider_agreements,
        }
    }

    /// Helper to check field completeness and update metrics
    fn check_field_completeness(
        &self,
        field_name: &str,
        is_complete: bool,
        field_completeness: &mut std::collections::HashMap<String, bool>,
        total_fields: &mut i32,
        complete_fields: &mut i32,
    ) {
        field_completeness.insert(field_name.to_string(), is_complete);
        *total_fields += 1;
        if is_complete {
            *complete_fields += 1;
        }
    }

    /// Calculate confidence score based on data quality metrics
    fn calculate_confidence_score(
        &self,
        anime: &crate::modules::anime::AnimeDetailed,
        quality: &DataQualityMetrics,
    ) -> f32 {
        let mut confidence = 0.0;

        // Weighted factors for confidence calculation (more conservative)
        confidence += quality.completeness_score * 0.35; // 35% weight on completeness
        confidence += quality.consistency_score * 0.25; // 25% weight on consistency
        confidence += quality.source_reliability * 0.25; // 25% weight on source reliability
        confidence += quality.freshness_score * 0.15; // 15% weight on freshness

        // Small bonuses for additional quality indicators (reduced)
        if !anime.provider_metadata.external_ids.is_empty() {
            confidence += 0.02; // Reduced from 0.05
        }

        if anime.score.is_some() {
            confidence += 0.02; // Reduced from 0.05
        }

        // Additional small penalty for missing important fields
        if anime.synopsis.is_none() {
            confidence -= 0.05;
        }
        if anime.genres.is_empty() {
            confidence -= 0.03;
        }

        // Ensure confidence is between 0.0 and 1.0
        confidence.min(1.0).max(0.0)
    }

    /// Calculate source reliability based on provider
    fn calculate_source_reliability(
        &self,
        provider: &crate::modules::provider::AnimeProvider,
    ) -> f32 {
        match provider {
            crate::modules::provider::AnimeProvider::AniList => 0.95, // Very reliable
            crate::modules::provider::AnimeProvider::Jikan => 0.90,   // Very reliable (MAL data)
            crate::modules::provider::AnimeProvider::Kitsu => 0.85,   // Good reliability
            crate::modules::provider::AnimeProvider::TMDB => 0.80,    // Good for movies/shows
            crate::modules::provider::AnimeProvider::AniDB => 0.85,   // Comprehensive but complex
        }
    }

    /// Calculate consistency score based on data quality indicators
    fn calculate_consistency_score(&self, anime: &crate::modules::anime::AnimeDetailed) -> f32 {
        let mut consistency = 0.5 as f32; // Base score

        // Check for data consistency indicators
        if anime.title.main.len() > 0 && anime.title.main.len() < 100 {
            consistency += 0.1; // Reasonable title length
        }

        // Check if episodes count makes sense
        if let Some(episodes) = anime.episodes {
            if episodes > 0 && episodes < 10000 {
                consistency += 0.1; // Reasonable episode count
            }
        }

        // Check if score is reasonable
        if let Some(score) = anime.score {
            if score >= 0.0 && score <= 10.0 {
                consistency += 0.1; // Score in valid range
            }
        }

        // Check if we have basic required fields filled
        if anime.synopsis.is_some() {
            consistency += 0.05;
        }
        if !anime.genres.is_empty() {
            consistency += 0.05;
        }
        if anime.image_url.is_some() {
            consistency += 0.05;
        }

        // Penalize if data seems inconsistent
        if anime.title.main.is_empty() {
            consistency -= 0.2;
        }

        // Multiple providers indicate better consistency
        if anime.provider_metadata.external_ids.len() > 1 {
            consistency += 0.1;
        }

        consistency.min(1.0).max(0.0)
    }

    /// Calculate freshness score based on how recent the data appears
    fn calculate_freshness_score(&self, anime: &crate::modules::anime::AnimeDetailed) -> f32 {
        let mut freshness = 0.6 as f32; // Base score for API data

        // Check if we have recent air dates
        if let Some(aired_from) = &anime.aired.from {
            let current_year = chrono::Utc::now().year();
            if aired_from.year() >= current_year - 1 {
                freshness += 0.2; // Recent anime
            } else if aired_from.year() >= current_year - 5 {
                freshness += 0.1; // Relatively recent
            } else if aired_from.year() < current_year - 20 {
                freshness -= 0.1; // Very old anime might have stale data
            }
        }

        // Having a score indicates the data is being maintained
        if anime.score.is_some() {
            freshness += 0.1;
        }

        // Multiple external IDs suggest actively maintained data
        if anime.provider_metadata.external_ids.len() > 1 {
            freshness += 0.1;
        }

        // Image URL suggests maintained visual data
        if anime.image_url.is_some() {
            freshness += 0.05;
        }

        freshness.min(1.0).max(0.0)
    }

    /// Extract provider sources from anime metadata
    fn extract_provider_sources(
        &self,
        anime: &crate::modules::anime::AnimeDetailed,
    ) -> Vec<crate::modules::provider::AnimeProvider> {
        let mut sources = vec![anime.provider_metadata.primary_provider.clone()];

        // Add additional providers that have external IDs for this anime
        for (provider, _) in &anime.provider_metadata.external_ids {
            if provider != &anime.provider_metadata.primary_provider && !sources.contains(provider)
            {
                sources.push(provider.clone());
            }
        }

        sources
    }

    /// Enhanced validation for multiple titles with comprehensive analysis
    pub async fn validate_titles_enhanced(
        &self,
        titles: Vec<String>,
        app: Option<AppHandle>,
    ) -> AppResult<EnhancedValidationResult> {
        let _timer = TimedOperation::new("validate_titles_enhanced");
        let start_time = std::time::Instant::now();
        let total_titles = titles.len();

        log_info!(
            "Starting enhanced validation for {} titles with comprehensive data analysis",
            total_titles
        );

        // Check for duplicate titles in input
        let unique_titles: std::collections::HashSet<_> = titles.iter().collect();
        if unique_titles.len() != titles.len() {
            log_warn!(
                "DUPLICATE TITLES DETECTED: Input has {} titles but only {} unique. Duplicates: {}",
                titles.len(),
                unique_titles.len(),
                titles.len() - unique_titles.len()
            );
        }

        let mut found = Vec::new();
        let mut not_found = Vec::new();
        let mut already_exists = Vec::new();
        let mut total_confidence = 0.0;
        let mut total_completeness = 0.0;
        let mut total_consistency = 0.0;
        let mut all_providers_used = std::collections::HashSet::new();

        // Process each title with enhanced validation and emit progress events
        for (index, title) in titles.iter().enumerate() {
            let _start_time = std::time::Instant::now();

            // Emit detailed step info for better UX
            if let Some(ref app_handle) = app {
                let step_info = ValidationStepInfo {
                    step: "searching".to_string(),
                    description: format!("Searching providers for '{}'", title),
                    title: title.clone(),
                    provider: "multi-provider".to_string(),
                    estimated_time_remaining: (total_titles - index - 1) as u32 * 2, // ~2s per title estimate
                };
                let _ = app_handle.emit("validation-step", step_info);
            }
            match self.validate_single_title_enhanced(title).await {
                EnhancedValidationSingleResult::Found(enhanced_anime) => {
                    total_confidence += enhanced_anime.confidence_score;
                    total_completeness += enhanced_anime.data_quality.completeness_score;
                    total_consistency += enhanced_anime.data_quality.consistency_score;

                    for provider in &enhanced_anime.provider_sources {
                        all_providers_used.insert(provider.clone());
                    }

                    found.push(enhanced_anime);
                }
                EnhancedValidationSingleResult::AlreadyExists(existing) => {
                    // No double lookup needed - already handled in validate_single_title_enhanced
                    already_exists.push(existing);
                }
                EnhancedValidationSingleResult::Failed(error) => {
                    not_found.push(error);
                }
            }

            // Emit progress update after processing each title
            if let Some(ref app_handle) = app {
                let current_found = found.len() as u32;
                let current_existing = already_exists.len() as u32;
                let current_failed = not_found.len() as u32;
                let current_progress = (index + 1) as u32;
                let percentage = (current_progress as f32 / total_titles as f32) * 100.0;

                let current_confidence = if current_found > 0 {
                    total_confidence / current_found as f32
                } else {
                    0.0
                };

                let progress = ValidationProgress {
                    current: current_progress,
                    total: total_titles as u32,
                    percentage,
                    current_title: title.clone(),
                    status: if index + 1 == total_titles {
                        "completed".to_string()
                    } else {
                        "processing".to_string()
                    },
                    found_count: current_found,
                    existing_count: current_existing,
                    failed_count: current_failed,
                    average_confidence: current_confidence,
                    providers_used: all_providers_used.len() as u32,
                };

                let _ = app_handle.emit("validation-progress", progress);
            }
        }

        let found_count = found.len();
        let average_confidence = if found_count > 0 {
            total_confidence / found_count as f32
        } else {
            0.0
        };
        let average_completeness = if found_count > 0 {
            total_completeness / found_count as f32
        } else {
            0.0
        };
        let average_consistency = if found_count > 0 {
            total_consistency / found_count as f32
        } else {
            0.0
        };

        // Determine most reliable provider (simplified)
        let most_reliable_provider =
            if all_providers_used.contains(&crate::modules::provider::AnimeProvider::AniList) {
                Some(crate::modules::provider::AnimeProvider::AniList)
            } else if all_providers_used.contains(&crate::modules::provider::AnimeProvider::Jikan) {
                Some(crate::modules::provider::AnimeProvider::Jikan)
            } else {
                all_providers_used.iter().next().cloned()
            };

        let data_quality_summary = DataQualitySummary {
            average_completeness,
            average_consistency,
            total_providers_used: all_providers_used.len() as u32,
            most_reliable_provider,
            fields_with_gaps: self.identify_common_gaps(&found),
        };

        // Emit final completion event to ensure 100% progress
        if let Some(ref app_handle) = app {
            let final_progress = ValidationProgress {
                current: total_titles as u32,
                total: total_titles as u32,
                percentage: 100.0,
                current_title: "Validation completed".to_string(),
                status: "completed".to_string(),
                found_count: found.len() as u32,
                existing_count: already_exists.len() as u32,
                failed_count: not_found.len() as u32,
                average_confidence,
                providers_used: all_providers_used.len() as u32,
            };
            let _ = app_handle.emit("validation-progress", final_progress);
        }

        log_info!(
            "Enhanced validation completed: {} found, {} existing, {} not found. Average confidence: {:.2}",
            found.len(), already_exists.len(), not_found.len(), average_confidence
        );

        // Calculate actual total from processed results (not input count)
        let actual_total = (found.len() + not_found.len() + already_exists.len()) as u32;

        log_info!(
            "Validation totals: Input={}, Processed={} (Found={}, Existing={}, Failed={})",
            total_titles,
            actual_total,
            found.len(),
            already_exists.len(),
            not_found.len()
        );

        // Debug log to help identify discrepancies
        let expected_total = found.len() + already_exists.len() + not_found.len();
        if total_titles != actual_total as usize {
            log_warn!(
                "INPUT vs PROCESSED MISMATCH: Input={} titles, but processed only {} results. Difference of {} titles may be due to duplicates or processing errors.",
                total_titles,
                actual_total,
                total_titles as i32 - actual_total as i32
            );
        }

        if expected_total != total_titles {
            log_warn!(
                "COUNTING DISCREPANCY: Input={} titles, Results: Found={}, Existing={}, Failed={}, Total={}. Extra/Missing={} titles",
                total_titles,
                found.len(),
                already_exists.len(),
                not_found.len(),
                expected_total,
                expected_total as i32 - total_titles as i32
            );
        }

        let validation_duration = start_time.elapsed();

        log_info!(
            "FINAL VALIDATION SUMMARY: Input={} -> Found={} new, Existing={}, Failed={}, Total processed={}, Duration={}ms",
            total_titles,
            found.len(),
            already_exists.len(),
            not_found.len(),
            actual_total,
            validation_duration.as_millis()
        );

        Ok(EnhancedValidationResult {
            found,
            not_found,
            already_exists,
            total: actual_total, // Use actual processed count, not input count
            average_confidence,
            data_quality_summary,
            validation_duration_ms: validation_duration.as_millis() as u64,
        })
    }

    /// Validate titles using enhanced processing
    pub async fn validate_titles_concurrent(
        &self,
        titles: Vec<String>,
        app: Option<AppHandle>,
    ) -> AppResult<EnhancedValidationResult> {
        // Use enhanced validation as the main implementation
        self.validate_titles_enhanced(titles, app).await
    }

    /// Identify commonly missing fields across validated anime
    fn identify_common_gaps(&self, found_anime: &[EnhancedValidatedAnime]) -> Vec<String> {
        let mut field_gaps = std::collections::HashMap::new();
        let total_count = found_anime.len();

        for anime in found_anime {
            for (field, is_complete) in &anime.data_quality.field_completeness {
                if !is_complete {
                    *field_gaps.entry(field.clone()).or_insert(0) += 1;
                }
            }
        }

        // Return fields that are missing in more than 30% of anime
        field_gaps
            .into_iter()
            .filter(|(_, missing_count)| *missing_count as f32 / total_count as f32 > 0.3)
            .map(|(field, _)| field)
            .collect()
    }
}

/// Result type for single validation operations
#[derive(Debug)]
pub enum ValidationSingleResult {
    Found(ValidatedAnime),
    AlreadyExists(ExistingAnime),
    Failed(ImportError),
}

/// Result type for enhanced single validation operations
#[derive(Debug)]
pub enum EnhancedValidationSingleResult {
    Found(EnhancedValidatedAnime),
    AlreadyExists(ExistingAnime),
    Failed(ImportError),
}
