use crate::modules::anime::AnimeRepository;
use crate::shared::errors::{AppError, AppResult};
use crate::shared::utils::logger::{LogContext, TimedOperation};
use crate::{log_info, log_warn};

use std::sync::Arc;

use super::types::{
    DataQualityMetrics, DataQualitySummary, EnhancedValidatedAnime, EnhancedValidationResult,
    ExistingAnime, ImportError, ValidatedAnime,
};
use crate::modules::provider::ProviderService;

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
        match self
            .provider_service
            .search_anime_comprehensive(query, 1)
            .await
        {
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
    ) -> Result<EnhancedValidatedAnime, ImportError> {
        let item_timer = TimedOperation::new("validate_single_title_enhanced");

        // STEP 1: Check database first (same as before)
        match self.anime_repo.find_by_title_variations(title).await {
            Ok(Some(existing_anime)) => {
                item_timer.finish();
                return Err(ImportError {
                    title: title.to_string(),
                    reason: format!("Already exists: {}", existing_anime.title.main),
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

                // STEP 3: Analyze data quality and generate comprehensive metrics
                let data_quality = self.analyze_anime_data_quality(&anime).await;
                let confidence_score = self.calculate_confidence_score(&anime, &data_quality);
                let provider_sources = self.extract_provider_sources(&anime);

                item_timer.finish();
                Ok(EnhancedValidatedAnime {
                    input_title: title.to_string(),
                    anime_data: anime,
                    data_quality,
                    provider_sources,
                    confidence_score,
                })
            }
            Ok(_) => {
                item_timer.finish();
                Err(ImportError {
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
                Err(ImportError {
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

        // For now, set consistency and freshness to reasonable defaults
        // In future iterations, we can enhance these by comparing data across providers
        let consistency_score = 0.85; // Based on comprehensive aggregation
        let freshness_score = 0.90; // Assume recent data from live APIs

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

        // Weighted factors for confidence calculation
        confidence += quality.completeness_score * 0.4; // 40% weight on completeness
        confidence += quality.consistency_score * 0.3; // 30% weight on consistency
        confidence += quality.source_reliability * 0.2; // 20% weight on source reliability
        confidence += quality.freshness_score * 0.1; // 10% weight on freshness

        // Bonus for having external IDs (indicates provider verification)
        if !anime.provider_metadata.external_ids.is_empty() {
            confidence += 0.05;
        }

        // Bonus for having score (indicates popularity/reliability)
        if anime.score.is_some() {
            confidence += 0.05;
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
    ) -> AppResult<EnhancedValidationResult> {
        let _timer = TimedOperation::new("validate_titles_enhanced");
        let total_titles = titles.len();

        log_info!(
            "Starting enhanced validation for {} titles with comprehensive data analysis",
            total_titles
        );

        let mut found = Vec::new();
        let mut not_found = Vec::new();
        let mut already_exists = Vec::new();
        let mut total_confidence = 0.0;
        let mut total_completeness = 0.0;
        let mut total_consistency = 0.0;
        let mut all_providers_used = std::collections::HashSet::new();

        // Process each title with enhanced validation
        for title in &titles {
            match self.validate_single_title_enhanced(title).await {
                Ok(enhanced_anime) => {
                    total_confidence += enhanced_anime.confidence_score;
                    total_completeness += enhanced_anime.data_quality.completeness_score;
                    total_consistency += enhanced_anime.data_quality.consistency_score;

                    for provider in &enhanced_anime.provider_sources {
                        all_providers_used.insert(provider.clone());
                    }

                    found.push(enhanced_anime);
                }
                Err(error) => {
                    if error.reason.contains("Already exists") {
                        // Handle existing anime case
                        match self.anime_repo.find_by_title_variations(title).await {
                            Ok(Some(existing_anime)) => {
                                already_exists.push(ExistingAnime {
                                    input_title: title.to_string(),
                                    matched_title: existing_anime.title.main.clone(),
                                    matched_field: "enhanced_validation".to_string(),
                                    anime: existing_anime,
                                });
                            }
                            _ => {
                                not_found.push(error);
                            }
                        }
                    } else {
                        not_found.push(error);
                    }
                }
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
            total_providers_used: all_providers_used.len(),
            most_reliable_provider,
            fields_with_gaps: self.identify_common_gaps(&found),
        };

        log_info!(
            "Enhanced validation completed: {} found, {} existing, {} not found. Average confidence: {:.2}",
            found.len(), already_exists.len(), not_found.len(), average_confidence
        );

        Ok(EnhancedValidationResult {
            found,
            not_found,
            already_exists,
            total: total_titles as u32,
            average_confidence,
            data_quality_summary,
        })
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
