use crate::log_info;
use crate::modules::anime::AnimeRepository;
use crate::modules::provider::ProviderManager;
use crate::shared::errors::AppResult;
use crate::shared::utils::logger::TimedOperation;

use super::concurrency_calculator::ConcurrencyCalculator;
use super::import_executor::ImportExecutor;
use super::progress_tracker::ProgressTracker;
use super::types::{
    ImportProgress, ImportResult, SkippedAnime, ValidatedAnime, ValidationProgress,
    ValidationResult,
};
use super::validation_service::{ValidationService, ValidationSingleResult};
use futures::{stream, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Atomic counters for progress tracking without lock contention
#[derive(Default)]
struct ProgressCounts {
    processed: AtomicUsize,
    imported: AtomicUsize,
    failed: AtomicUsize,
    skipped: AtomicUsize,
}

/// Orchestrates the import workflow using focused components
#[derive(Clone)]
pub struct ImportCoordinator {
    validation_service: ValidationService,
    import_executor: ImportExecutor,
    progress_tracker: ProgressTracker,
}

impl ImportCoordinator {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        provider_manager: Arc<Mutex<ProviderManager>>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Self {
        let validation_service =
            ValidationService::new(anime_repo.clone(), provider_manager.clone());
        let import_executor = ImportExecutor::new(anime_repo.clone());
        let progress_tracker = ProgressTracker::new(app_handle);

        Self {
            validation_service,
            import_executor,
            progress_tracker,
        }
    }

    /// Optimized validation using new components with existing logic
    pub async fn validate_anime_titles(&self, titles: Vec<String>) -> AppResult<ValidationResult> {
        let _timer = TimedOperation::new("validate_anime_titles");
        let total_titles = titles.len();

        log_info!("Starting validation for {} titles", total_titles);

        let progress_tracker = self
            .progress_tracker
            .clone()
            .with_batch_config(total_titles);
        let mut found = Vec::new();
        let mut not_found = Vec::new();
        let mut already_exists = Vec::new();
        let mut events_emitted = 0;
        let mut last_emitted_percentage = 0;

        // Emit initial progress
        progress_tracker.emit_validation_progress(ValidationProgress {
            current: 0,
            total: total_titles,
            current_title: "Starting validation...".to_string(),
            processed: 0,
            found_count: 0,
            existing_count: 0,
            failed_count: 0,
        });

        // Small delay to ensure frontend receives initial event
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Process titles using the ValidationService
        for (index, title) in titles.iter().enumerate() {
            let result = self.validation_service.validate_single_title(title).await;

            match result {
                ValidationSingleResult::Found(validated) => found.push(validated),
                ValidationSingleResult::AlreadyExists(existing) => already_exists.push(existing),
                ValidationSingleResult::Failed(error) => not_found.push(error),
            }

            // Emit batched progress updates using the ProgressTracker
            let processed = index + 1;
            if progress_tracker.should_emit_validation_progress(
                processed,
                total_titles,
                &mut last_emitted_percentage,
                false,                     // is_initial
                processed == total_titles, // is_final
            ) {
                events_emitted += 1;
                progress_tracker.emit_validation_progress(ValidationProgress {
                    current: processed,
                    total: total_titles,
                    current_title: if processed < total_titles {
                        format!("Processing... ({}/{})", processed, total_titles)
                    } else {
                        "Validation completed".to_string()
                    },
                    processed,
                    found_count: found.len(),
                    existing_count: already_exists.len(),
                    failed_count: not_found.len(),
                });
            }
        }

        let result = ValidationResult {
            found: found.clone(),
            not_found: not_found.clone(),
            already_exists: already_exists.clone(),
            total: u32::try_from(titles.len()).unwrap_or(u32::MAX),
        };

        log_info!(
            "Validation completed: {} found, {} already exist, {} not found. Events emitted: {} (vs {} items processed - {:.1}% reduction)",
            found.len(),
            already_exists.len(),
            not_found.len(),
            events_emitted,
            total_titles,
            if total_titles > 0 {
                100.0 - (events_emitted as f64 / total_titles as f64 * 100.0)
            } else {
                0.0
            }
        );

        Ok(result)
    }

    /// Import validated anime using new components with existing concurrency logic
    pub async fn import_validated_anime(
        &self,
        validated_anime: Vec<ValidatedAnime>,
    ) -> AppResult<ImportResult> {
        let _timer = TimedOperation::new("import_validated_anime");
        let total_count = validated_anime.len();

        log_info!("Starting validated anime import for {} items", total_count);

        // Calculate dynamic concurrency for database operations (reused from existing)
        let db_concurrency = ConcurrencyCalculator::calculate_db_concurrency();
        log_info!("Using dynamic DB concurrency: {}", db_concurrency);

        let imported = Arc::new(Mutex::new(Vec::new()));
        let failed = Arc::new(Mutex::new(Vec::new()));
        let skipped = Arc::new(Mutex::new(Vec::new()));
        let counts = Arc::new(ProgressCounts::default());

        // Emit initial progress
        self.progress_tracker.emit_import_progress(ImportProgress {
            current: 0,
            total: total_count,
            current_title: "Starting validated anime import...".to_string(),
            processed: 0,
            imported_count: 0,
            failed_count: 0,
            skipped_count: 0,
        });

        // Process all validated anime using buffer_unordered for better performance
        let results = stream::iter(validated_anime.into_iter().enumerate().map(
            |(index, validated)| {
                let import_executor = self.import_executor.clone();
                let progress_tracker = self.progress_tracker.clone();
                let counts_clone = counts.clone();

                async move {
                    let current_index = index + 1;
                    let anime_title = validated.anime_data.title.main.clone();

                    // Emit progress for current item (optional - could be moved to result processing)
                    let current_processed = counts_clone.processed.load(Ordering::Relaxed);
                    let imported_len = counts_clone.imported.load(Ordering::Relaxed);
                    let failed_len = counts_clone.failed.load(Ordering::Relaxed);
                    let skipped_len = counts_clone.skipped.load(Ordering::Relaxed);

                    progress_tracker.emit_import_progress(ImportProgress {
                        current: current_index,
                        total: total_count,
                        current_title: format!("Importing: {}", anime_title),
                        processed: current_processed,
                        imported_count: imported_len,
                        failed_count: failed_len,
                        skipped_count: skipped_len,
                    });

                    // Execute the import and return result with metadata
                    let result = import_executor.import_single_validated(&validated).await;
                    (index, validated, result)
                }
            },
        ))
        .buffer_unordered(db_concurrency)
        .collect::<Vec<_>>()
        .await;

        // Process all results and update collections + counters
        for (_index, validated, result) in results {
            match result {
                Ok(imported_anime) => {
                    imported.lock().await.push(imported_anime);
                    counts.imported.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    // Check if this should be treated as skipped or failed
                    if e.reason.contains("Already exists") || e.reason.contains("Duplicate") {
                        let (external_id, provider) =
                            ValidationService::get_primary_external_info(&validated.anime_data);
                        skipped.lock().await.push(SkippedAnime {
                            title: validated.input_title,
                            external_id,
                            provider,
                            reason: e.reason,
                        });
                        counts.skipped.fetch_add(1, Ordering::Relaxed);
                    } else {
                        failed.lock().await.push(e);
                        counts.failed.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            counts.processed.fetch_add(1, Ordering::Relaxed);
        }

        // Extract results
        let imported_results = imported.lock().await.clone();
        let failed_results = failed.lock().await.clone();
        let skipped_results = skipped.lock().await.clone();
        let total = imported_results.len() + skipped_results.len() + failed_results.len();

        // Emit final completion progress
        self.progress_tracker.emit_import_progress(ImportProgress {
            current: total_count,
            total: total_count,
            current_title: "Validated import completed".to_string(),
            processed: total,
            imported_count: imported_results.len(),
            failed_count: failed_results.len(),
            skipped_count: skipped_results.len(),
        });

        log_info!(
            "Import validated anime completed: {} imported, {} skipped, {} failed",
            imported_results.len(),
            skipped_results.len(),
            failed_results.len()
        );

        Ok(ImportResult {
            imported: imported_results,
            skipped: skipped_results,
            failed: failed_results,
            total: u32::try_from(total).unwrap_or(u32::MAX),
        })
    }

    /// Import anime batch - simplified version using validation + import
    pub async fn import_anime_batch(&self, titles: Vec<String>) -> AppResult<ImportResult> {
        let _timer = TimedOperation::new("import_anime_batch");
        let total_count = titles.len();

        log_info!(
            "Starting batch import for {} titles using new components",
            total_count
        );

        // Step 1: Validate all titles first
        let validation_result = self.validate_anime_titles(titles).await?;

        // Step 2: Import the validated anime
        let import_result = self.import_validated_anime(validation_result.found).await?;

        // Step 3: Combine results (found + already_exists = total processed)
        let mut final_result = import_result;

        // Add already existing anime as skipped
        for existing in validation_result.already_exists {
            let (external_id, provider) =
                ValidationService::get_primary_external_info(&existing.anime);
            final_result.skipped.push(SkippedAnime {
                title: existing.input_title,
                external_id,
                provider,
                reason: "Already exists in database".to_string(),
            });
        }

        // Add failed validations to failed imports
        final_result.failed.extend(validation_result.not_found);

        // Update total count
        final_result.total = total_count as u32;

        log_info!(
            "Batch import completed: {} imported, {} skipped, {} failed",
            final_result.imported.len(),
            final_result.skipped.len(),
            final_result.failed.len()
        );

        Ok(final_result)
    }
}
