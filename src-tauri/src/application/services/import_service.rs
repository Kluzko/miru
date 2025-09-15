use super::provider_manager::ProviderManager;
use crate::domain::repositories::AnimeRepository;
use crate::domain::value_objects::AnimeProvider;
use crate::shared::errors::{AppError, AppResult};
use crate::shared::utils::logger::{LogContext, TimedOperation};
use crate::{log_error, log_info, log_warn};
use specta::Type;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

/// Import service with dynamic concurrency optimization
///
/// This service automatically adjusts concurrency limits based on:
/// - Provider-specific rate limits (e.g., MAL: ~3 req/s, AniList: ~1.5 req/s)
/// - System resources (CPU cores for DB operations)
/// - Safety factors to prevent rate limit violations
///
/// Key improvements over fixed concurrency:
/// - API calls: Calculated from provider rate limits with 80% safety factor
/// - DB operations: Scaled based on CPU cores (2x multiplier)
/// - Provider-aware batching for mixed provider scenarios
/// - Comprehensive logging of calculated limits
#[derive(Clone)]
pub struct ImportService {
    anime_repo: Arc<dyn AnimeRepository>,
    provider_manager: Arc<tokio::sync::Mutex<ProviderManager>>,
}

impl ImportService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        provider_manager: Arc<tokio::sync::Mutex<ProviderManager>>,
    ) -> Self {
        Self {
            anime_repo,
            provider_manager,
        }
    }

    /// Calculate optimal concurrency for API requests based on provider rate limits
    async fn calculate_api_concurrency(&self, provider: &AnimeProvider) -> usize {
        const SAFETY_FACTOR: f64 = 0.8; // Use 80% of theoretical max to be conservative
        const MIN_CONCURRENCY: usize = 1;
        const MAX_CONCURRENCY: usize = 10;

        let provider_manager = self.provider_manager.lock().await;

        match provider_manager.get_provider_rate_limit(provider) {
            Some(rate_info) => {
                // Base concurrency on requests per second with safety factor
                let theoretical_max =
                    (rate_info.requests_per_second * SAFETY_FACTOR).ceil() as usize;
                let optimal = theoretical_max.max(MIN_CONCURRENCY).min(MAX_CONCURRENCY);

                log_info!(
                    "Calculated API concurrency for {:?}: {} (rate: {:.2} req/s, safety factor: {:.1})",
                    provider, optimal, rate_info.requests_per_second, SAFETY_FACTOR
                );

                optimal
            }
            None => {
                log_warn!(
                    "No rate limit info for provider {:?}, using default concurrency: {}",
                    provider,
                    MIN_CONCURRENCY
                );
                MIN_CONCURRENCY
            }
        }
    }

    /// Calculate optimal concurrency for database operations based on system resources
    fn calculate_db_concurrency() -> usize {
        const MIN_DB_CONCURRENCY: usize = 2;
        const MAX_DB_CONCURRENCY: usize = 20;
        const DB_CONCURRENCY_PER_CPU: usize = 2;

        // Base DB concurrency on CPU cores (assuming connection pool can handle it)
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4); // Default to 4 if can't detect

        let optimal = (cpu_count * DB_CONCURRENCY_PER_CPU)
            .max(MIN_DB_CONCURRENCY)
            .min(MAX_DB_CONCURRENCY);

        log_info!(
            "Calculated DB concurrency: {} (CPUs: {}, multiplier: {}x)",
            optimal,
            cpu_count,
            DB_CONCURRENCY_PER_CPU
        );

        optimal
    }

    /// Create provider-aware semaphores for mixed batch processing
    async fn create_provider_semaphores(
        &self,
        providers: &[AnimeProvider],
    ) -> HashMap<AnimeProvider, Arc<tokio::sync::Semaphore>> {
        let mut semaphores = HashMap::new();

        for provider in providers {
            let concurrency = self.calculate_api_concurrency(provider).await;
            let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
            semaphores.insert(provider.clone(), semaphore);
        }

        semaphores
    }

    /// Group validated anime by their provider for optimized batch processing
    fn group_anime_by_provider(
        validated_anime: &[ValidatedAnime],
    ) -> HashMap<AnimeProvider, Vec<&ValidatedAnime>> {
        let mut groups = HashMap::new();

        for anime in validated_anime {
            let provider = anime.anime_data.provider_metadata.primary_provider.clone();
            groups.entry(provider).or_insert_with(Vec::new).push(anime);
        }

        for (provider, animes) in &groups {
            log_info!("Provider {:?}: {} anime to process", provider, animes.len());
        }

        groups
    }

    /// Helper method to get primary external ID and provider from anime
    fn get_primary_external_info(
        anime: &crate::domain::entities::AnimeDetailed,
    ) -> (String, crate::domain::value_objects::AnimeProvider) {
        let provider = anime.provider_metadata.primary_provider.clone();
        let external_id = anime
            .provider_metadata
            .get_external_id(&provider)
            .cloned()
            .unwrap_or_else(|| "0".to_string());
        (external_id, provider)
    }

    /// Helper method to check if external ID is valid
    fn is_valid_external_id(external_id: &str) -> bool {
        crate::shared::utils::Validator::is_valid_external_id(external_id)
    }

    /// Search for anime using provider manager with fallback
    async fn search_anime_multi_provider(
        &self,
        query: &str,
    ) -> AppResult<Vec<crate::domain::entities::AnimeDetailed>> {
        let mut provider_manager = self.provider_manager.lock().await;

        match provider_manager.search_anime(query, 1).await {
            Ok(results) if !results.is_empty() => {
                LogContext::search_operation(query, Some("provider_manager"), Some(results.len()));
                Ok(results)
            }
            Ok(_) => {
                LogContext::search_operation(query, Some("provider_manager"), Some(0));
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

    /// Import anime batch with progress reporting and dynamic concurrency optimization
    pub async fn import_anime_batch(
        &self,
        titles: Vec<String>,
        app_handle: Option<tauri::AppHandle>,
        _cancellation_token: Option<CancellationToken>,
    ) -> AppResult<ImportResult> {
        let _timer = TimedOperation::new("import_anime_batch");

        let total_count = titles.len();
        log_info!(
            "Starting dynamic concurrency batch import for {} titles",
            total_count
        );

        // Calculate dynamic concurrency limits
        let primary_provider = {
            let provider_manager = self.provider_manager.lock().await;
            provider_manager.get_primary_provider()
        };

        let api_concurrency = self.calculate_api_concurrency(&primary_provider).await;
        let db_concurrency = Self::calculate_db_concurrency();

        log_info!(
            "Dynamic concurrency limits - API: {}, DB: {} for provider: {:?}",
            api_concurrency,
            db_concurrency,
            primary_provider
        );

        let imported = Arc::new(Mutex::new(Vec::new()));
        let failed = Arc::new(Mutex::new(Vec::new()));
        let skipped = Arc::new(Mutex::new(Vec::new()));
        let processed_count = Arc::new(AtomicUsize::new(0));

        // Emit initial progress
        if let Some(ref app) = app_handle {
            let progress = ImportProgress {
                current: 0,
                total: total_count,
                current_title: "Starting import...".to_string(),
                processed: 0,
                imported_count: 0,
                failed_count: 0,
                skipped_count: 0,
            };
            let _ = app.emit("import_progress", &progress);
        }

        // Use dynamically calculated semaphore limits
        let api_semaphore = Arc::new(tokio::sync::Semaphore::new(api_concurrency));
        let mut join_set = JoinSet::new();

        // Spawn all tasks first
        for (_index, title) in titles.into_iter().enumerate() {
            let title_clone = title.clone();
            let api_semaphore_clone = api_semaphore.clone();
            let service = self.clone();
            let imported_clone = imported.clone();
            let failed_clone = failed.clone();
            let skipped_clone = skipped.clone();
            let processed_count_clone = processed_count.clone();
            let app_handle_clone = app_handle.clone();

            join_set.spawn(async move {
                let _api_permit = api_semaphore_clone.acquire().await.unwrap();

                // Emit progress for current item
                if let Some(ref app) = app_handle_clone {
                    let current_processed = processed_count_clone.load(Ordering::Relaxed);
                    let imported_len = imported_clone.lock().await.len();
                    let failed_len = failed_clone.lock().await.len();
                    let skipped_len = skipped_clone.lock().await.len();

                    let progress = ImportProgress {
                        current: current_processed + 1,
                        total: total_count,
                        current_title: title_clone.clone(),
                        processed: current_processed,
                        imported_count: imported_len,
                        failed_count: failed_len,
                        skipped_count: skipped_len,
                    };
                    let _ = app.emit("import_progress", &progress);
                }

                // Use optimized DB-first import approach
                let current_item_index = processed_count_clone.load(Ordering::Relaxed) + 1;
                let progress_context = Some((current_item_index, total_count));

                let result = service
                    .import_single_anime_optimized(
                        &title_clone,
                        app_handle_clone.as_ref(),
                        progress_context,
                    )
                    .await;

                // Handle result and update collections
                match result {
                    ImportItemResult::Imported(anime) => {
                        imported_clone.lock().await.push(anime);
                    }
                    ImportItemResult::Skipped(anime) => {
                        skipped_clone.lock().await.push(anime);
                    }
                    ImportItemResult::Failed(error) => {
                        failed_clone.lock().await.push(error);
                    }
                }

                // REPLACED OLD INEFFICIENT CODE WITH OPTIMIZED VERSION ABOVE
                processed_count_clone.fetch_add(1, Ordering::Relaxed);

                // Emit final progress for this item
                if let Some(ref app) = app_handle_clone {
                    let current_processed = processed_count_clone.load(Ordering::Relaxed);
                    let imported_len = imported_clone.lock().await.len();
                    let failed_len = failed_clone.lock().await.len();
                    let skipped_len = skipped_clone.lock().await.len();

                    let progress = ImportProgress {
                        current: current_processed,
                        total: total_count,
                        current_title: format!("Processed '{}'", title_clone),
                        processed: current_processed,
                        imported_count: imported_len,
                        failed_count: failed_len,
                        skipped_count: skipped_len,
                    };
                    let _ = app.emit("import_progress", &progress);
                }
            });
        }

        // Stream task completions as they arrive
        let mut successful_tasks = 0;
        let mut failed_tasks = 0;
        let total_spawned_tasks = join_set.len();

        log_info!(
            "Starting streaming task processing: {} tasks spawned",
            total_spawned_tasks
        );

        while let Some(result) = join_set.join_next().await {
            let pending_tasks = join_set.len();

            match result {
                Ok(_) => {
                    successful_tasks += 1;
                    log_info!(
                        "Task completed successfully - Progress: {}/{} completed, {} pending",
                        successful_tasks + failed_tasks,
                        total_spawned_tasks,
                        pending_tasks
                    );
                }
                Err(e) => {
                    failed_tasks += 1;
                    log_error!(
                        "Background task failed: {:?} - Progress: {}/{} completed, {} pending",
                        e,
                        successful_tasks + failed_tasks,
                        total_spawned_tasks,
                        pending_tasks
                    );
                    // Add to failed results if task panicked
                    failed.lock().await.push(ImportError {
                        title: "Unknown".to_string(),
                        reason: format!("Task panicked: {:?}", e),
                    });
                }
            }
        }

        log_info!(
            "Import batch completed: {} successful, {} failed tasks",
            successful_tasks,
            failed_tasks
        );

        // Extract results from Arc<Mutex<>>
        let imported_results = imported.lock().await.clone();
        let failed_results = failed.lock().await.clone();
        let skipped_results = skipped.lock().await.clone();

        let total = imported_results.len() + skipped_results.len() + failed_results.len();

        // Emit final completion progress
        if let Some(ref app) = app_handle {
            let progress = ImportProgress {
                current: total_count,
                total: total_count,
                current_title: "Import completed".to_string(),
                processed: total,
                imported_count: imported_results.len(),
                failed_count: failed_results.len(),
                skipped_count: skipped_results.len(),
            };
            let _ = app.emit("import_progress", &progress);
        }

        Ok(ImportResult {
            imported: imported_results,
            skipped: skipped_results,
            failed: failed_results,
            total: total as u32,
        })
    }

    pub async fn import_from_csv(&self, csv_content: &str) -> AppResult<ImportResult> {
        let mut reader = csv::Reader::from_reader(csv_content.as_bytes());
        let mut titles = Vec::new();

        // Extract titles from CSV
        for result in reader.records() {
            match result {
                Ok(record) => {
                    if let Some(first_field) = record.get(0) {
                        if !first_field.trim().is_empty() {
                            titles.push(first_field.trim().to_string());
                        }
                    }
                }
                Err(e) => {
                    return Err(AppError::InvalidInput(format!("Invalid CSV: {}", e)));
                }
            }
        }

        // Import by titles
        if !titles.is_empty() {
            self.import_anime_batch(titles, None, None).await
        } else {
            Err(AppError::InvalidInput(
                "No valid data found in CSV".to_string(),
            ))
        }
    }

    /// Optimized validation with DB-first lookup and batched progress events
    pub async fn validate_anime_titles(
        &self,
        titles: Vec<String>,
        app_handle: Option<&tauri::AppHandle>,
    ) -> AppResult<ValidationResult> {
        let _timer = TimedOperation::new("validate_anime_titles");

        let total_titles = titles.len();
        log_info!("Starting validation for {} titles", total_titles);

        let mut found = Vec::new();
        let mut not_found = Vec::new();
        let mut already_exists = Vec::new();

        // Batching configuration
        let batch_size = std::cmp::max(1, total_titles / 50);
        let min_percentage_change = 1;
        let mut last_emitted_percentage = 0;
        let mut events_emitted = 0;

        log_info!(
            "Validation batching configured: batch_size={}, min_percentage_change={}%",
            batch_size,
            min_percentage_change
        );

        // Helper function to emit progress with batching logic
        let emit_progress = |app: &tauri::AppHandle,
                             current: usize,
                             total: usize,
                             current_title: String,
                             processed: usize,
                             found_count: usize,
                             existing_count: usize,
                             failed_count: usize,
                             events_emitted: &mut usize,
                             last_emitted_percentage: &mut usize,
                             is_initial: bool,
                             is_final: bool|
         -> bool {
            let current_percentage = if total > 0 {
                (processed * 100) / total
            } else {
                0
            };
            let should_emit_percentage = current_percentage
                .saturating_sub(*last_emitted_percentage)
                >= min_percentage_change;
            let should_emit_batch = processed % batch_size == 0;

            let should_emit = is_initial || is_final || should_emit_percentage || should_emit_batch;

            if should_emit {
                let progress = ValidationProgress {
                    current,
                    total,
                    current_title,
                    processed,
                    found_count,
                    existing_count,
                    failed_count,
                };

                match app.emit("validation_progress", &progress) {
                    Ok(_) => {
                        *events_emitted += 1;
                        *last_emitted_percentage = current_percentage;
                        true
                    }
                    Err(e) => {
                        log_error!("Failed to emit validation progress: {}", e);
                        false
                    }
                }
            } else {
                false
            }
        };

        // Emit initial progress
        if let Some(app) = app_handle {
            emit_progress(
                app,
                0,
                total_titles,
                "Starting validation...".to_string(),
                0,
                0,
                0,
                0,
                &mut events_emitted,
                &mut last_emitted_percentage,
                true,  // is_initial
                false, // is_final
            );

            // Small delay to ensure frontend receives initial event
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // Process titles sequentially with optimized DB-first lookup
        for (index, title) in titles.iter().enumerate() {
            let item_timer = TimedOperation::new("validate_single_title");

            // STEP 1: Check database first using title variations (same as optimized import)
            match self.anime_repo.find_by_title_variations(title).await {
                Ok(Some(existing_anime)) => {
                    already_exists.push(ExistingAnime {
                        input_title: title.clone(),
                        matched_title: existing_anime.title.main.clone(),
                        matched_field: "title_match".to_string(),
                        anime: existing_anime,
                    });
                    item_timer.finish();
                }
                Ok(None) => {
                    // STEP 2: Search providers only if not found in DB
                    match self.search_anime_multi_provider(title).await {
                        Ok(anime_list) if !anime_list.is_empty() => {
                            let anime = anime_list.into_iter().next().unwrap();
                            let (external_id, provider) = Self::get_primary_external_info(&anime);

                            if Self::is_valid_external_id(&external_id) {
                                // STEP 3: Double-check by external_id to avoid duplicates
                                match self
                                    .anime_repo
                                    .find_by_external_id(&provider, &external_id)
                                    .await
                                {
                                    Ok(Some(existing)) => {
                                        already_exists.push(ExistingAnime {
                                            input_title: title.clone(),
                                            matched_title: existing.title.main.clone(),
                                            matched_field: format!("{:?}_id", provider),
                                            anime: existing,
                                        });
                                    }
                                    Ok(None) => {
                                        found.push(ValidatedAnime {
                                            input_title: title.clone(),
                                            anime_data: anime,
                                        });
                                    }
                                    Err(e) => {
                                        log_warn!(
                                            "External ID check failed for '{}': {}",
                                            title,
                                            e
                                        );
                                        not_found.push(ImportError {
                                            title: title.clone(),
                                            reason: format!(
                                                "Database error during external ID check: {}",
                                                e
                                            ),
                                        });
                                    }
                                }
                            } else {
                                log_warn!("Invalid external ID for '{}': {}", title, external_id);
                                // Still add to found list as it might be importable
                                found.push(ValidatedAnime {
                                    input_title: title.clone(),
                                    anime_data: anime,
                                });
                            }
                        }
                        Ok(_) => {
                            not_found.push(ImportError {
                                title: title.clone(),
                                reason: "No results found on any provider".to_string(),
                            });
                        }
                        Err(e) => {
                            LogContext::error_with_context(
                                &e,
                                &format!("Provider search failed for '{}'", title),
                            );
                            not_found.push(ImportError {
                                title: title.clone(),
                                reason: format!("Provider search failed: {}", e),
                            });
                        }
                    }
                    item_timer.finish();
                }
                Err(e) => {
                    log_warn!("Database lookup failed for '{}': {}", title, e);
                    // Continue to provider lookup as fallback
                    match self.search_anime_multi_provider(title).await {
                        Ok(anime_list) if !anime_list.is_empty() => {
                            let anime = anime_list.into_iter().next().unwrap();
                            let (external_id, provider) = Self::get_primary_external_info(&anime);

                            if Self::is_valid_external_id(&external_id) {
                                match self
                                    .anime_repo
                                    .find_by_external_id(&provider, &external_id)
                                    .await
                                {
                                    Ok(Some(existing)) => {
                                        already_exists.push(ExistingAnime {
                                            input_title: title.clone(),
                                            matched_title: existing.title.main.clone(),
                                            matched_field: format!("{:?}_id", provider),
                                            anime: existing,
                                        });
                                    }
                                    Ok(None) => {
                                        found.push(ValidatedAnime {
                                            input_title: title.clone(),
                                            anime_data: anime,
                                        });
                                    }
                                    Err(e) => {
                                        log_warn!(
                                            "External ID check failed for '{}': {}",
                                            title,
                                            e
                                        );
                                        not_found.push(ImportError {
                                            title: title.clone(),
                                            reason: format!(
                                                "Database error during external ID check: {}",
                                                e
                                            ),
                                        });
                                    }
                                }
                            } else {
                                found.push(ValidatedAnime {
                                    input_title: title.clone(),
                                    anime_data: anime,
                                });
                            }
                        }
                        Ok(_) => {
                            not_found.push(ImportError {
                                title: title.clone(),
                                reason: "No results found on any provider".to_string(),
                            });
                        }
                        Err(e) => {
                            LogContext::error_with_context(
                                &e,
                                &format!("Provider search failed for '{}'", title),
                            );
                            not_found.push(ImportError {
                                title: title.clone(),
                                reason: format!("Provider search failed: {}", e),
                            });
                        }
                    }
                    item_timer.finish();
                }
            }

            // Emit batched progress updates
            if let Some(app) = app_handle {
                let processed = index + 1;
                emit_progress(
                    app,
                    processed,
                    total_titles,
                    if processed < total_titles {
                        format!("Processing... ({}/{})", processed, total_titles)
                    } else {
                        "Validation completed".to_string()
                    },
                    processed,
                    found.len(),
                    already_exists.len(),
                    not_found.len(),
                    &mut events_emitted,
                    &mut last_emitted_percentage,
                    false,                     // is_initial
                    processed == total_titles, // is_final
                );
            }
        }

        // Emit final completion progress (if not already emitted)
        if let Some(app) = app_handle {
            emit_progress(
                app,
                total_titles,
                total_titles,
                "Validation completed".to_string(),
                total_titles,
                found.len(),
                already_exists.len(),
                not_found.len(),
                &mut events_emitted,
                &mut last_emitted_percentage,
                false, // is_initial
                true,  // is_final
            );
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

    /// Import validated anime to database with dynamic concurrency optimization
    pub async fn import_validated_anime(
        &self,
        validated_anime: Vec<ValidatedAnime>,
        app_handle: Option<tauri::AppHandle>,
    ) -> AppResult<ImportResult> {
        let _timer = TimedOperation::new("import_validated_anime");

        let total_count = validated_anime.len();
        log_info!(
            "Starting dynamic concurrency validated anime import for {} items",
            total_count
        );

        // Calculate dynamic concurrency for database operations
        let db_concurrency = Self::calculate_db_concurrency();
        log_info!("Using dynamic DB concurrency: {}", db_concurrency);

        // Use Arc<Mutex<>> for thread-safe collections as in other optimized functions
        let imported = Arc::new(Mutex::new(Vec::new()));
        let failed = Arc::new(Mutex::new(Vec::new()));
        let skipped = Arc::new(Mutex::new(Vec::new()));
        let processed_count = Arc::new(AtomicUsize::new(0));

        // Emit initial progress
        if let Some(ref app) = app_handle {
            let progress = ImportProgress {
                current: 0,
                total: total_count,
                current_title: "Starting validated anime import...".to_string(),
                processed: 0,
                imported_count: 0,
                failed_count: 0,
                skipped_count: 0,
            };
            let _ = app.emit("import_progress", &progress);
        }

        // Use dynamically calculated semaphore for database operations
        let db_semaphore = Arc::new(tokio::sync::Semaphore::new(db_concurrency));
        let mut join_set = JoinSet::new();

        for (index, validated) in validated_anime.into_iter().enumerate() {
            let validated_clone = validated;
            let db_semaphore_clone = db_semaphore.clone();
            let service = self.clone();
            let imported_clone = imported.clone();
            let failed_clone = failed.clone();
            let skipped_clone = skipped.clone();
            let processed_count_clone = processed_count.clone();
            let app_handle_clone = app_handle.clone();

            join_set.spawn(async move {
                let _db_permit = db_semaphore_clone.acquire().await.unwrap();
                let item_timer = TimedOperation::new("import_single_validated_anime");

                let current_index = index + 1;
                let anime_title = &validated_clone.anime_data.title.main;

                // Emit progress for current item
                if let Some(ref app) = app_handle_clone {
                    let current_processed = processed_count_clone.load(Ordering::Relaxed);
                    let imported_len = imported_clone.lock().await.len();
                    let failed_len = failed_clone.lock().await.len();
                    let skipped_len = skipped_clone.lock().await.len();

                    let progress = ImportProgress {
                        current: current_index,
                        total: total_count,
                        current_title: format!("Importing: {}", anime_title),
                        processed: current_processed,
                        imported_count: imported_len,
                        failed_count: failed_len,
                        skipped_count: skipped_len,
                    };
                    let _ = app.emit("import_progress", &progress);
                }

                log_info!(
                    "Processing {}/{}: '{}'",
                    current_index,
                    total_count,
                    anime_title
                );

                // Get external ID and provider for duplicate checking
                let (external_id, provider) =
                    ImportService::get_primary_external_info(&validated_clone.anime_data);

                // STEP 1: Double-check for duplicates using external ID (defensive programming)
                if ImportService::is_valid_external_id(&external_id) {
                    match service
                        .anime_repo
                        .find_by_external_id(&provider, &external_id)
                        .await
                    {
                        Ok(Some(_existing_anime)) => {
                            log_info!(
                                "Skipping '{}' - already exists in database with {} ID: {}",
                                anime_title,
                                match provider {
                                    crate::domain::value_objects::AnimeProvider::Jikan => "MAL",
                                    crate::domain::value_objects::AnimeProvider::AniList =>
                                        "AniList",
                                    _ => "Unknown",
                                },
                                external_id
                            );
                            skipped_clone.lock().await.push(SkippedAnime {
                                title: anime_title.clone(),
                                external_id: external_id.clone(),
                                provider: provider.clone(),
                                reason: "Already exists in database".to_string(),
                            });
                            item_timer.finish();
                            processed_count_clone.fetch_add(1, Ordering::Relaxed);
                            return;
                        }
                        Ok(None) => {
                            // Continue with import
                            log_info!(
                                "External ID check passed for '{}', proceeding with save",
                                anime_title
                            );
                        }
                        Err(e) => {
                            log_warn!("External ID check failed for '{}': {}", anime_title, e);
                            // Continue with import as fallback
                        }
                    }
                }

                // STEP 2: Save to database with proper error handling
                match service.anime_repo.save(&validated_clone.anime_data).await {
                    Ok(saved_anime) => {
                        let (saved_external_id, saved_provider) =
                            ImportService::get_primary_external_info(&saved_anime);

                        log_info!(
                            "Successfully imported '{}' from {} with ID: {}",
                            saved_anime.title.main,
                            match saved_provider {
                                crate::domain::value_objects::AnimeProvider::Jikan => "MAL",
                                crate::domain::value_objects::AnimeProvider::AniList => "AniList",
                                _ => "Unknown",
                            },
                            saved_external_id
                        );

                        imported_clone.lock().await.push(ImportedAnime {
                            title: saved_anime.title.main.clone(),
                            primary_external_id: saved_external_id,
                            provider: saved_provider,
                            id: saved_anime.id,
                        });
                    }
                    Err(e) => {
                        LogContext::error_with_context(
                            &e,
                            &format!("Failed to save validated anime '{}'", anime_title),
                        );

                        // Check if this is a unique constraint violation (duplicate)
                        let error_msg = e.to_string();
                        if error_msg.contains("UNIQUE constraint failed")
                            || error_msg.contains("duplicate")
                        {
                            log_info!(
                                "Duplicate detected during save for '{}', treating as skipped",
                                anime_title
                            );
                            skipped_clone.lock().await.push(SkippedAnime {
                                title: validated_clone.input_title.clone(),
                                external_id: external_id.clone(),
                                provider: provider.clone(),
                                reason: "Duplicate detected during save".to_string(),
                            });
                        } else {
                            failed_clone.lock().await.push(ImportError {
                                title: validated_clone.input_title.clone(),
                                reason: format!("Database error: {}", e),
                            });
                        }
                    }
                }

                item_timer.finish();
                processed_count_clone.fetch_add(1, Ordering::Relaxed);

                // Emit final progress for this item
                if let Some(ref app) = app_handle_clone {
                    let current_processed = processed_count_clone.load(Ordering::Relaxed);
                    let imported_len = imported_clone.lock().await.len();
                    let failed_len = failed_clone.lock().await.len();
                    let skipped_len = skipped_clone.lock().await.len();

                    let progress = ImportProgress {
                        current: current_processed,
                        total: total_count,
                        current_title: format!("Processed: {}", anime_title),
                        processed: current_processed,
                        imported_count: imported_len,
                        failed_count: failed_len,
                        skipped_count: skipped_len,
                    };
                    let _ = app.emit("import_progress", &progress);
                }
            });
        }

        // Stream validated anime import completions as they arrive
        let mut successful_tasks = 0;
        let mut failed_tasks = 0;
        let total_spawned_tasks = join_set.len();

        log_info!(
            "Starting streaming validated import processing: {} tasks spawned",
            total_spawned_tasks
        );

        while let Some(result) = join_set.join_next().await {
            let pending_tasks = join_set.len();

            match result {
                Ok(_) => {
                    successful_tasks += 1;
                    log_info!(
                        "Validated import task completed successfully - Progress: {}/{} completed, {} pending",
                        successful_tasks + failed_tasks,
                        total_spawned_tasks,
                        pending_tasks
                    );
                }
                Err(e) => {
                    failed_tasks += 1;
                    log_error!(
                        "Validated import task failed: {:?} - Progress: {}/{} completed, {} pending",
                        e,
                        successful_tasks + failed_tasks,
                        total_spawned_tasks,
                        pending_tasks
                    );
                    // Add to failed results if task panicked
                    failed.lock().await.push(ImportError {
                        title: "Unknown validated anime".to_string(),
                        reason: format!("Task panicked: {:?}", e),
                    });
                }
            }
        }

        log_info!(
            "Validated anime import completed: {} successful, {} failed tasks",
            successful_tasks,
            failed_tasks
        );

        // Extract results from Arc<Mutex<>>
        let imported_results = imported.lock().await.clone();
        let failed_results = failed.lock().await.clone();
        let skipped_results = skipped.lock().await.clone();

        let total = imported_results.len() + skipped_results.len() + failed_results.len();

        // Emit final completion progress
        if let Some(ref app) = app_handle {
            let progress = ImportProgress {
                current: total_count,
                total: total_count,
                current_title: "Validated import completed".to_string(),
                processed: total,
                imported_count: imported_results.len(),
                failed_count: failed_results.len(),
                skipped_count: skipped_results.len(),
            };
            let _ = app.emit("import_progress", &progress);
        }

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

    /// Import a single anime item with proper error handling
    async fn import_single_anime(&self, title: &str) -> ImportItemResult {
        let item_timer = TimedOperation::new("import_single_anime");

        // Search for anime
        let anime_list = match self.search_anime_multi_provider(title).await {
            Ok(list) if !list.is_empty() => list,
            Ok(_) => {
                item_timer.finish();
                return ImportItemResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: "No results found on any provider".to_string(),
                });
            }
            Err(e) => {
                LogContext::error_with_context(&e, &format!("Search failed for '{}'", title));
                item_timer.finish();
                return ImportItemResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: format!("Search failed: {}", e),
                });
            }
        };

        let anime = anime_list.into_iter().next().unwrap();
        let (external_id, provider) = Self::get_primary_external_info(&anime);

        if !Self::is_valid_external_id(&external_id) {
            log_warn!(
                "Invalid external ID for '{}': {}",
                anime.title.main,
                external_id
            );
            item_timer.finish();
            return ImportItemResult::Failed(ImportError {
                title: title.to_string(),
                reason: "Invalid external ID".to_string(),
            });
        }

        // Check if already exists
        match self
            .anime_repo
            .find_by_external_id(&provider, &external_id)
            .await
        {
            Ok(Some(_)) => {
                log_info!(
                    "Skipping '{}' - already exists in database with {} ID: {}",
                    anime.title.main,
                    match provider {
                        crate::domain::value_objects::AnimeProvider::Jikan => "MAL",
                        crate::domain::value_objects::AnimeProvider::AniList => "AniList",
                        _ => "Unknown",
                    },
                    external_id
                );
                item_timer.finish();
                return ImportItemResult::Skipped(SkippedAnime {
                    title: anime.title.main.clone(),
                    external_id: external_id.clone(),
                    provider: provider.clone(),
                    reason: "Already exists in database".to_string(),
                });
            }
            Ok(None) => {
                // Save to database
                match self.anime_repo.save(&anime).await {
                    Ok(saved_anime) => {
                        log_info!(
                            "Successfully imported '{}' from {} with ID: {}",
                            saved_anime.title.main,
                            match provider {
                                crate::domain::value_objects::AnimeProvider::Jikan => "MAL",
                                crate::domain::value_objects::AnimeProvider::AniList => "AniList",
                                _ => "Unknown",
                            },
                            external_id
                        );
                        item_timer.finish();
                        return ImportItemResult::Imported(ImportedAnime {
                            title: saved_anime.title.main.clone(),
                            primary_external_id: external_id.clone(),
                            provider: provider.clone(),
                            id: saved_anime.id,
                        });
                    }
                    Err(e) => {
                        LogContext::error_with_context(
                            &e,
                            &format!("Failed to save '{}'", anime.title.main),
                        );
                        item_timer.finish();
                        return ImportItemResult::Failed(ImportError {
                            title: title.to_string(),
                            reason: format!("Failed to save: {}", e),
                        });
                    }
                }
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    &format!("Database error for '{}'", anime.title.main),
                );
                item_timer.finish();
                return ImportItemResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: format!("Database error: {}", e),
                });
            }
        }
    }

    /// Optimized import with DB-first lookup to avoid unnecessary provider calls
    async fn import_single_anime_optimized(
        &self,
        title: &str,
        app_handle: Option<&tauri::AppHandle>,
        progress_context: Option<(usize, usize)>, // (current, total) for progress reporting
    ) -> ImportItemResult {
        let item_timer = TimedOperation::new("import_single_anime_optimized");

        // Emit progress for DB lookup phase
        if let (Some(app), Some((current, total))) = (app_handle, progress_context) {
            let progress = ImportProgress {
                current,
                total,
                current_title: format!("Checking DB: {}", title),
                processed: current,
                imported_count: 0, // These will be updated by caller
                failed_count: 0,
                skipped_count: 0,
            };
            let _ = app.emit("import_progress", &progress);
        }

        // STEP 1: Check if anime already exists in database first (avoid provider calls)
        match self.anime_repo.find_by_title_variations(title).await {
            Ok(Some(existing_anime)) => {
                log_info!("Found '{}' in database, skipping provider lookup", title);
                item_timer.finish();
                return ImportItemResult::Skipped(SkippedAnime {
                    title: title.to_string(),
                    external_id: existing_anime
                        .provider_metadata
                        .get_external_id(&existing_anime.provider_metadata.primary_provider)
                        .unwrap_or(&"unknown".to_string())
                        .clone(),
                    provider: existing_anime.provider_metadata.primary_provider.clone(),
                    reason: "Already exists in database".to_string(),
                });
            }
            Ok(None) => {
                log_info!("'{}' not found in database, checking providers", title);
                // Continue to provider lookup
            }
            Err(e) => {
                log_warn!("Database lookup failed for '{}': {}", title, e);
                // Continue to provider lookup as fallback
            }
        }

        // Emit progress for provider lookup phase
        if let (Some(app), Some((current, total))) = (app_handle, progress_context) {
            let progress = ImportProgress {
                current,
                total,
                current_title: format!("Searching providers: {}", title),
                processed: current,
                imported_count: 0,
                failed_count: 0,
                skipped_count: 0,
            };
            let _ = app.emit("import_progress", &progress);
        }

        // STEP 2: Search providers only if not found in DB
        let anime_list = match self.search_anime_multi_provider(title).await {
            Ok(list) if !list.is_empty() => list,
            Ok(_) => {
                item_timer.finish();
                return ImportItemResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: "No results found on any provider".to_string(),
                });
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    &format!("Provider search failed for '{}'", title),
                );
                item_timer.finish();
                return ImportItemResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: format!("Provider search failed: {}", e),
                });
            }
        };

        let anime = anime_list.into_iter().next().unwrap();
        let (external_id, provider) = Self::get_primary_external_info(&anime);

        if !Self::is_valid_external_id(&external_id) {
            log_warn!(
                "Invalid external ID for '{}': {}",
                anime.title.main,
                external_id
            );
            item_timer.finish();
            return ImportItemResult::Failed(ImportError {
                title: title.to_string(),
                reason: "Invalid external ID".to_string(),
            });
        }

        // STEP 3: Double-check by external_id (in case title search missed variants)
        match self
            .anime_repo
            .find_by_external_id(&provider, &external_id)
            .await
        {
            Ok(Some(_)) => {
                log_info!("Found '{}' by external ID {}, skipping", title, external_id);
                item_timer.finish();
                return ImportItemResult::Skipped(SkippedAnime {
                    title: anime.title.main.clone(),
                    external_id: external_id.clone(),
                    provider: provider.clone(),
                    reason: "Already exists in database".to_string(),
                });
            }
            Ok(None) => {
                // Save to database
                match self.anime_repo.save(&anime).await {
                    Ok(_) => {
                        log_info!("Successfully imported: {}", anime.title.main);
                        item_timer.finish();
                        return ImportItemResult::Imported(ImportedAnime {
                            title: anime.title.main.clone(),
                            primary_external_id: external_id,
                            provider,
                            id: anime.id,
                        });
                    }
                    Err(e) => {
                        LogContext::error_with_context(
                            &e,
                            &format!("Failed to save anime {}", anime.title.main),
                        );
                        item_timer.finish();
                        return ImportItemResult::Failed(ImportError {
                            title: title.to_string(),
                            reason: format!("Database save failed: {}", e),
                        });
                    }
                }
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    &format!("External ID check failed for '{}'", title),
                );
                item_timer.finish();
                return ImportItemResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: format!("Database check failed: {}", e),
                });
            }
        }
    }
}

/// Result type for individual import operations
#[derive(Debug)]
enum ImportItemResult {
    Imported(ImportedAnime),
    Skipped(SkippedAnime),
    Failed(ImportError),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ImportResult {
    pub imported: Vec<ImportedAnime>,
    pub failed: Vec<ImportError>,
    pub skipped: Vec<SkippedAnime>,
    pub total: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ImportedAnime {
    pub title: String,
    pub primary_external_id: String,
    pub provider: crate::domain::value_objects::AnimeProvider,
    pub id: uuid::Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ImportError {
    pub title: String,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct SkippedAnime {
    pub title: String,
    pub external_id: String,
    pub provider: crate::domain::value_objects::AnimeProvider,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ValidationResult {
    pub found: Vec<ValidatedAnime>,
    pub not_found: Vec<ImportError>,
    pub already_exists: Vec<ExistingAnime>,
    pub total: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ValidatedAnime {
    pub input_title: String,
    pub anime_data: crate::domain::entities::AnimeDetailed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ExistingAnime {
    pub input_title: String,
    pub matched_title: String,
    pub matched_field: String,
    pub anime: crate::domain::entities::AnimeDetailed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ImportProgress {
    pub current: usize,
    pub total: usize,
    pub current_title: String,
    pub processed: usize,
    pub imported_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ValidationProgress {
    pub current: usize,
    pub total: usize,
    pub current_title: String,
    pub processed: usize,
    pub found_count: usize,
    pub existing_count: usize,
    pub failed_count: usize,
}
