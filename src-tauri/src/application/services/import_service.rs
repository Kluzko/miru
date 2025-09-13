use super::provider_manager::ProviderManager;
use crate::domain::repositories::AnimeRepository;
use crate::shared::errors::{AppError, AppResult};
use specta::Type;
use std::sync::Arc;
use tauri::Emitter;

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
                println!(
                    "DEBUG: Found {} results for '{}' via provider manager",
                    results.len(),
                    query
                );
                Ok(results)
            }
            Ok(_) => {
                println!("DEBUG: No results found for '{}'", query);
                Ok(vec![])
            }
            Err(e) => {
                println!(
                    "DEBUG: Provider manager search failed for '{}': {}",
                    query, e
                );
                Err(e)
            }
        }
    }

    pub async fn import_anime_batch(&self, titles: Vec<String>) -> AppResult<ImportResult> {
        self.import_anime_batch_with_progress(titles, None).await
    }

    pub async fn import_anime_batch_with_progress(
        &self,
        titles: Vec<String>,
        app_handle: Option<tauri::AppHandle>,
    ) -> AppResult<ImportResult> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use tokio::sync::Mutex;

        let imported = Arc::new(Mutex::new(Vec::new()));
        let failed = Arc::new(Mutex::new(Vec::new()));
        let skipped = Arc::new(Mutex::new(Vec::new()));
        let processed_count = Arc::new(AtomicUsize::new(0));

        let total_count = titles.len();

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

        // Use semaphore to limit concurrent requests while respecting rate limits
        let semaphore = Arc::new(tokio::sync::Semaphore::new(3)); // Max 3 concurrent requests
        let mut handles = Vec::new();

        for (index, title) in titles.into_iter().enumerate() {
            let title_clone = title.clone();
            let semaphore_clone = semaphore.clone();
            let service = self.clone();
            let imported_clone = imported.clone();
            let failed_clone = failed.clone();
            let skipped_clone = skipped.clone();
            let processed_count_clone = processed_count.clone();
            let app_handle_clone = app_handle.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

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

                match service.search_anime_multi_provider(&title_clone).await {
                    Ok(anime_list) if !anime_list.is_empty() => {
                        let anime = anime_list.into_iter().next().unwrap();
                        // Check if already exists using proper provider handling
                        let (external_id, provider) = Self::get_primary_external_info(&anime);

                        if Self::is_valid_external_id(&external_id) {
                            match service
                                .anime_repo
                                .find_by_external_id(&provider, &external_id)
                                .await
                            {
                                Ok(Some(_)) => {
                                    println!(
                                    "DEBUG: Skipping '{}' - already exists in database with {} ID: {}",
                                    anime.title.main,
                                    match provider {
                                        crate::domain::value_objects::AnimeProvider::Jikan => "MAL",
                                        crate::domain::value_objects::AnimeProvider::AniList => "AniList",
                                        _ => "Unknown"
                                    },
                                    external_id
                                );
                                    skipped_clone.lock().await.push(SkippedAnime {
                                        title: anime.title.main.clone(),
                                        external_id: external_id.clone(),
                                        provider: provider.clone(),
                                        reason: "Already exists in database".to_string(),
                                    });
                                }
                                Ok(None) => {
                                    // Save to database
                                    match service.anime_repo.save(&anime).await {
                                        Ok(saved_anime) => {
                                            println!(
                                            "DEBUG: Successfully imported '{}' from {} with ID: {}",
                                            saved_anime.title.main,
                                            match provider {
                                                crate::domain::value_objects::AnimeProvider::Jikan => "MAL",
                                                crate::domain::value_objects::AnimeProvider::AniList => "AniList",
                                                _ => "Unknown"
                                            },
                                            external_id
                                        );
                                            imported_clone.lock().await.push(ImportedAnime {
                                                title: saved_anime.title.main.clone(),
                                                primary_external_id: external_id.clone(),
                                                provider: provider.clone(),
                                                id: saved_anime.id,
                                            });
                                        }
                                        Err(e) => {
                                            println!(
                                                "DEBUG: Failed to save '{}': {}",
                                                anime.title.main, e
                                            );
                                            failed_clone.lock().await.push(ImportError {
                                                title: title_clone.clone(),
                                                reason: format!("Failed to save: {}", e),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!(
                                        "DEBUG: Database error for '{}': {}",
                                        anime.title.main, e
                                    );
                                    failed_clone.lock().await.push(ImportError {
                                        title: title_clone.clone(),
                                        reason: format!("Database error: {}", e),
                                    });
                                }
                            }
                        } else {
                            println!(
                                "DEBUG: Invalid external ID for '{}': {}",
                                anime.title.main, external_id
                            );
                            failed_clone.lock().await.push(ImportError {
                                title: title_clone.clone(),
                                reason: "Invalid external ID".to_string(),
                            });
                        }
                    }
                    Ok(_) => {
                        failed_clone.lock().await.push(ImportError {
                            title: title_clone.clone(),
                            reason: "No results found on any provider".to_string(),
                        });
                    }
                    Err(e) => {
                        failed_clone.lock().await.push(ImportError {
                            title: title_clone.clone(),
                            reason: format!("Search failed: {}", e),
                        });
                    }
                }

                // Increment processed count
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

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let _ = handle.await;
        }

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
            self.import_anime_batch(titles).await
        } else {
            Err(AppError::InvalidInput(
                "No valid data found in CSV".to_string(),
            ))
        }
    }

    /// Simple validation without optimizations
    pub async fn validate_anime_titles(&self, titles: Vec<String>) -> AppResult<ValidationResult> {
        let mut found = Vec::new();
        let mut not_found = Vec::new();
        let mut already_exists = Vec::new();

        // Process titles sequentially
        for title in titles.iter() {
            match self.anime_repo.find_by_title_variations(title).await {
                Ok(Some(existing_anime)) => {
                    already_exists.push(ExistingAnime {
                        input_title: title.clone(),
                        matched_title: existing_anime.title.to_string(),
                        matched_field: "title_match".to_string(),
                        anime: existing_anime,
                    });
                }
                Ok(None) => {
                    // Try API search
                    match self.search_anime_multi_provider(&title).await {
                        Ok(anime_list) if !anime_list.is_empty() => {
                            let anime = anime_list.into_iter().next().unwrap();

                            // Check if we already have this anime by external ID
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
                                            matched_title: existing.title.to_string(),
                                            matched_field: format!("{:?}_id", provider),
                                            anime: existing,
                                        });
                                        continue;
                                    }
                                    _ => {
                                        found.push(ValidatedAnime {
                                            input_title: title.clone(),
                                            anime_data: anime,
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
                        _ => {
                            not_found.push(ImportError {
                                title: title.clone(),
                                reason: "No results found".to_string(),
                            });
                        }
                    }
                }
                Err(e) => {
                    not_found.push(ImportError {
                        title: title.clone(),
                        reason: format!("Database error: {}", e),
                    });
                }
            }
        }

        Ok(ValidationResult {
            found,
            not_found,
            already_exists,
            total: u32::try_from(titles.len()).unwrap_or(u32::MAX),
        })
    }

    /// Import validated anime to database
    pub async fn import_validated_anime(
        &self,
        validated_anime: Vec<ValidatedAnime>,
    ) -> AppResult<ImportResult> {
        let mut imported = Vec::new();
        let mut failed = Vec::new();
        let skipped = Vec::new();

        for validated in &validated_anime {
            match self.anime_repo.save(&validated.anime_data).await {
                Ok(saved_anime) => {
                    let (saved_external_id, saved_provider) =
                        Self::get_primary_external_info(&saved_anime);

                    imported.push(ImportedAnime {
                        title: saved_anime.title.to_string(),
                        primary_external_id: saved_external_id,
                        provider: saved_provider,
                        id: saved_anime.id,
                    });
                }
                Err(e) => {
                    failed.push(ImportError {
                        title: validated.input_title.clone(),
                        reason: format!("Database error: {}", e),
                    });
                }
            }
        }

        Ok(ImportResult {
            imported,
            failed,
            skipped,
            total: u32::try_from(validated_anime.len()).unwrap_or(u32::MAX),
        })
    }
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
