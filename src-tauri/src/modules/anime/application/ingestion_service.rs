/// AnimeIngestionService - Unified anime creation pipeline
///
/// This service provides a single, consistent entry point for creating anime records,
/// regardless of the source (manual import, relations discovery, franchise discovery).
///
/// Key responsibilities:
/// - Validate anime data quality
/// - Enhance incomplete data from multiple providers
/// - Calculate quality metrics and tier
/// - Save to database via AnimeService (ensuring proper score calculation)
/// - Queue async enrichment jobs for low-quality data
///
/// This replaces the duplicate logic previously scattered across:
/// - data_import module (manual import)
/// - anime_relations_service (relations discovery)
use crate::modules::anime::application::service::AnimeService;
use crate::modules::anime::domain::entities::anime_detailed::AnimeDetailed;
use crate::modules::data_import::domain::services::import_components::{
    data_enhancement_service::DataEnhancementService, types::DataQualityMetrics,
    validation_service::ValidationService,
};
use crate::modules::provider::{AnimeProvider, ProviderService};
use crate::shared::errors::AppResult;
use crate::{log_debug, log_info};
use futures::future;
use std::sync::Arc;

/// Source from which anime is being ingested
#[derive(Debug, Clone)]
pub enum AnimeSource {
    /// User manually importing anime via UI
    ManualImport { title: String },

    /// System discovered anime through franchise relations
    RelationDiscovery {
        anilist_id: u32,
        relation_type: String,
        source_anime_id: String,
    },

    /// System discovered anime through franchise name
    FranchiseDiscovery { franchise_name: String },

    /// Direct anime data (already fetched from provider)
    DirectData {
        anime: AnimeDetailed,
        context: String, // e.g., "AniList relation", "Manual search"
    },
}

impl AnimeSource {
    /// Get a human-readable description for logging
    pub fn description(&self) -> String {
        match self {
            AnimeSource::ManualImport { title } => format!("Manual import: '{}'", title),
            AnimeSource::RelationDiscovery {
                relation_type,
                source_anime_id,
                ..
            } => format!(
                "Relation discovery: {} from {}",
                relation_type, source_anime_id
            ),
            AnimeSource::FranchiseDiscovery { franchise_name } => {
                format!("Franchise discovery: '{}'", franchise_name)
            }
            AnimeSource::DirectData { anime, context } => {
                format!("Direct data: '{}' ({})", anime.title.main, context)
            }
        }
    }

    /// Get the title for validation/search purposes
    pub fn get_title(&self) -> Option<String> {
        match self {
            AnimeSource::ManualImport { title } => Some(title.clone()),
            AnimeSource::FranchiseDiscovery { franchise_name } => Some(franchise_name.clone()),
            AnimeSource::DirectData { anime, .. } => Some(anime.title.main.clone()),
            AnimeSource::RelationDiscovery { .. } => None, // Will fetch by ID
        }
    }
}

/// Options controlling ingestion behavior
#[derive(Debug, Clone)]
pub struct IngestionOptions {
    /// Queue deep enrichment as background job?
    pub enrich_async: bool,

    /// Skip if anime already exists in database?
    pub skip_duplicates: bool,

    /// Discover and ingest related anime?
    pub fetch_relations: bool,

    /// Priority for queued jobs (High/Normal/Low)
    pub priority: JobPriority,

    /// Skip provider fetching (use existing comprehensive data)
    pub skip_provider_fetch: bool,
}

impl Default for IngestionOptions {
    fn default() -> Self {
        Self {
            enrich_async: false,
            skip_duplicates: true,
            fetch_relations: false,
            priority: JobPriority::Normal,
            skip_provider_fetch: false,
        }
    }
}

/// Priority levels for background jobs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobPriority {
    High = 1,
    Normal = 5,
    Low = 10,
}

/// Result of anime ingestion operation
#[derive(Debug, Clone)]
pub struct IngestionResult {
    /// The saved anime record
    pub anime: AnimeDetailed,

    /// Data quality score (0.0 to 1.0)
    pub quality_score: f32,

    /// Was async enrichment queued?
    pub enrichment_queued: bool,

    /// Improvements made during ingestion
    pub improvements_made: Vec<String>,

    /// Whether this was a new insert or existing anime
    pub was_new: bool,
}

/// Unified anime ingestion service
pub struct AnimeIngestionService {
    validation_service: Arc<ValidationService>,
    enhancement_service: Arc<DataEnhancementService>,
    anime_service: Arc<AnimeService>,
    provider_service: Arc<ProviderService>,
    job_repository: Arc<dyn crate::modules::jobs::domain::repository::JobRepository>,
}

impl AnimeIngestionService {
    pub fn new(
        validation_service: Arc<ValidationService>,
        enhancement_service: Arc<DataEnhancementService>,
        anime_service: Arc<AnimeService>,
        provider_service: Arc<ProviderService>,
        job_repository: Arc<dyn crate::modules::jobs::domain::repository::JobRepository>,
    ) -> Self {
        Self {
            validation_service,
            enhancement_service,
            anime_service,
            provider_service,
            job_repository,
        }
    }

    /// Main ingestion pipeline - handles all anime creation
    ///
    /// Pipeline stages:
    /// 1. Validation & Fetching - Find anime via providers (with built-in duplicate detection)
    /// 2. Enhancement - Fill data gaps from multiple providers
    /// 3. Quality Calculation & Save - Calculate tier/scores via AnimeService.create_anime()
    /// 4. Async Enrichment - Queue background job if quality is low (optional)
    /// 5. Relations Discovery - Queue job to find related anime (optional)
    pub async fn ingest_anime(
        &self,
        source: AnimeSource,
        options: IngestionOptions,
    ) -> AppResult<IngestionResult> {
        log_info!("Starting anime ingestion: {}", source.description());

        // STAGE 1: Validation & Fetching
        // Note: fetch_anime_data already checks for duplicates via ValidationService,
        // which returns AlreadyExists variant. If an existing anime is found,
        // it's returned here and we proceed to enhance/save it (updating existing record).
        let anime_data = self.fetch_anime_data(&source, &options).await?;

        // STAGE 2: Basic Enhancement (quick)
        let quality_metrics = self.calculate_quality_metrics(&anime_data);
        let enhancement_result = self
            .enhancement_service
            .enhance_anime_data(&anime_data, &quality_metrics, options.skip_provider_fetch)
            .await?;

        log_debug!(
            "Enhanced '{}': quality {:.2} -> {:.2}",
            anime_data.title.main,
            enhancement_result.quality_score_before,
            enhancement_result.quality_score_after
        );

        // STAGE 3: Quality Calculation + Save
        // Check if anime already exists to set was_new correctly
        let was_new = if options.skip_duplicates {
            self.anime_service
                .get_anime_by_id(&enhancement_result.enhanced_anime.id)
                .await?
                .is_none()
        } else {
            true // If not checking duplicates, consider it new
        };

        // This calls AnimeService.create_anime() which internally calls anime.update_scores()
        // This ensures tier, quality_metrics, and composite_score are properly calculated
        let saved_anime = self
            .anime_service
            .create_anime(&enhancement_result.enhanced_anime)
            .await?;

        log_info!(
            "Successfully ingested '{}' with tier {:?}, quality {:.2}",
            saved_anime.title.main,
            saved_anime.tier,
            enhancement_result.quality_score_after
        );

        // STAGE 4: Async Deep Enrichment (if needed)
        let enrichment_queued =
            if options.enrich_async && enhancement_result.quality_score_after < 0.8 {
                log_info!(
                    "Anime '{}' quality {:.2} < 0.8, queueing enrichment job",
                    saved_anime.title.main,
                    enhancement_result.quality_score_after
                );

                let job = crate::modules::jobs::domain::entities::Job::enrichment(
                    saved_anime.id,
                    options.priority as i32,
                );

                match self.job_repository.enqueue(job).await {
                    Ok(job_record) => {
                        log_info!(
                            "Enrichment job queued for anime '{}' (job_id: {})",
                            saved_anime.title.main,
                            job_record.id
                        );
                        true
                    }
                    Err(e) => {
                        log_debug!("Failed to queue enrichment job: {}", e);
                        false
                    }
                }
            } else {
                false
            };

        // STAGE 5: Discover Relations (if requested)
        if options.fetch_relations {
            log_info!(
                "Queueing relations discovery job for '{}'",
                saved_anime.title.main
            );

            let job = crate::modules::jobs::domain::entities::Job::relations_discovery(
                saved_anime.id,
                5, // Normal priority
            );

            match self.job_repository.enqueue(job).await {
                Ok(job_record) => {
                    log_info!(
                        "Relations discovery job queued for anime '{}' (job_id: {})",
                        saved_anime.title.main,
                        job_record.id
                    );
                }
                Err(e) => {
                    log_debug!("Failed to queue relations discovery job: {}", e);
                }
            }
        }

        Ok(IngestionResult {
            anime: saved_anime,
            quality_score: enhancement_result.quality_score_after,
            enrichment_queued,
            improvements_made: enhancement_result.improvements_made,
            was_new,
        })
    }

    /// Batch ingestion for multiple anime (efficient for relations discovery)
    ///
    /// Processes anime in parallel for better performance
    pub async fn ingest_batch(
        &self,
        sources: Vec<AnimeSource>,
        options: IngestionOptions,
    ) -> Vec<AppResult<IngestionResult>> {
        log_info!("Starting batch ingestion of {} anime", sources.len());

        // Process all ingestions in parallel
        let futures = sources.into_iter().map(|source| {
            let options_clone = options.clone();
            async move { self.ingest_anime(source, options_clone).await }
        });

        // Wait for all to complete
        let results = future::join_all(futures).await;

        let (success_count, failure_count) =
            results.iter().fold(
                (0, 0),
                |(s, f), r| {
                    if r.is_ok() {
                        (s + 1, f)
                    } else {
                        (s, f + 1)
                    }
                },
            );

        log_info!(
            "Batch ingestion completed: {} succeeded, {} failed",
            success_count,
            failure_count
        );

        results
    }

    // ========================================================================
    // PRIVATE HELPER METHODS
    // ========================================================================

    /// Fetch anime data based on source type
    async fn fetch_anime_data(
        &self,
        source: &AnimeSource,
        _options: &IngestionOptions,
    ) -> AppResult<AnimeDetailed> {
        match source {
            AnimeSource::ManualImport { title } => {
                // Use validation service to search providers
                self.fetch_by_title(title).await
            }

            AnimeSource::RelationDiscovery { anilist_id, .. } => {
                // Fetch directly from AniList by ID
                self.fetch_by_provider_id(*anilist_id, AnimeProvider::AniList)
                    .await
            }

            AnimeSource::FranchiseDiscovery { franchise_name } => {
                // Search by franchise name
                self.fetch_by_title(franchise_name).await
            }

            AnimeSource::DirectData { anime, .. } => {
                // Already have the data, just return it
                Ok(anime.clone())
            }
        }
    }

    /// Fetch anime by title using validation service
    async fn fetch_by_title(&self, title: &str) -> AppResult<AnimeDetailed> {
        use crate::modules::data_import::domain::services::import_components::validation_service::EnhancedValidationSingleResult;

        match self
            .validation_service
            .validate_single_title_enhanced(title)
            .await
        {
            EnhancedValidationSingleResult::Found(enhanced) => Ok(enhanced.anime_data),
            EnhancedValidationSingleResult::AlreadyExists(existing) => Ok(existing.anime),
            EnhancedValidationSingleResult::Failed(error) => {
                Err(crate::shared::errors::AppError::NotFound(format!(
                    "Anime '{}' not found: {}",
                    title, error.reason
                )))
            }
        }
    }

    /// Fetch anime by provider ID (for relations discovery)
    async fn fetch_by_provider_id(
        &self,
        provider_id: u32,
        provider: AnimeProvider,
    ) -> AppResult<AnimeDetailed> {
        match self
            .provider_service
            .get_anime_by_id(&provider_id.to_string(), provider)
            .await?
        {
            Some(anime) => Ok(anime),
            None => Err(crate::shared::errors::AppError::NotFound(format!(
                "Anime with {:?} ID {} not found",
                provider, provider_id
            ))),
        }
    }

    /// Calculate quality metrics for anime data
    fn calculate_quality_metrics(&self, anime: &AnimeDetailed) -> DataQualityMetrics {
        let mut field_completeness = std::collections::HashMap::new();
        let mut complete_fields = 0;
        let total_fields = 11;

        // Check each field
        let fields = [
            ("title.main", !anime.title.main.is_empty()),
            ("title.english", anime.title.english.is_some()),
            ("title.japanese", anime.title.japanese.is_some()),
            ("synopsis", anime.synopsis.is_some()),
            ("genres", !anime.genres.is_empty()),
            ("studios", !anime.studios.is_empty()),
            ("score", anime.score.is_some()),
            ("age_restriction", anime.age_restriction.is_some()),
            ("aired_from", anime.aired.from.is_some()),
            ("image_url", anime.image_url.is_some()),
            ("episodes", anime.episodes.is_some()),
        ];

        for (field_name, is_complete) in fields.iter() {
            field_completeness.insert(field_name.to_string(), *is_complete);
            if *is_complete {
                complete_fields += 1;
            }
        }

        let completeness_score = complete_fields as f32 / total_fields as f32;
        let consistency_score = self.calculate_consistency_score(anime);

        DataQualityMetrics {
            completeness_score,
            consistency_score,
            freshness_score: 1.0,    // Assume fresh since just fetched
            source_reliability: 0.9, // High reliability from providers
            field_completeness,
            provider_agreements: std::collections::HashMap::new(),
        }
    }

    /// Calculate consistency score
    fn calculate_consistency_score(&self, anime: &AnimeDetailed) -> f32 {
        let mut score: f32 = 1.0;

        // Check for inconsistencies
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

        if anime.title.main.is_empty() {
            score -= 0.3;
        }

        score.max(0.0)
    }

    /// Calculate overall quality score
    fn calculate_quality_score(&self, anime: &AnimeDetailed) -> f32 {
        let metrics = self.calculate_quality_metrics(anime);
        (metrics.completeness_score
            + metrics.consistency_score
            + metrics.freshness_score
            + metrics.source_reliability)
            / 4.0
    }
}

// ========================================================================
// TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anime_source_description() {
        let source = AnimeSource::ManualImport {
            title: "Attack on Titan".to_string(),
        };
        assert_eq!(source.description(), "Manual import: 'Attack on Titan'");

        let source = AnimeSource::RelationDiscovery {
            anilist_id: 12345,
            relation_type: "SEQUEL".to_string(),
            source_anime_id: "abc-123".to_string(),
        };
        assert!(source.description().contains("SEQUEL"));
    }

    #[test]
    fn test_ingestion_options_default() {
        let options = IngestionOptions::default();
        assert!(!options.enrich_async);
        assert!(options.skip_duplicates);
        assert!(!options.fetch_relations);
        assert_eq!(options.priority, JobPriority::Normal);
    }

    #[test]
    fn test_job_priority_values() {
        assert_eq!(JobPriority::High as i32, 1);
        assert_eq!(JobPriority::Normal as i32, 5);
        assert_eq!(JobPriority::Low as i32, 10);
    }
}
