use crate::domain::repositories::AnimeRepository;
use crate::infrastructure::external::jikan::JikanClient;
use crate::shared::errors::{AppError, AppResult};
use specta::Type;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub struct ImportService {
    anime_repo: Arc<dyn AnimeRepository>,
    jikan_client: Arc<JikanClient>,
}

impl ImportService {
    pub fn new(anime_repo: Arc<dyn AnimeRepository>, jikan_client: Arc<JikanClient>) -> Self {
        Self {
            anime_repo,
            jikan_client,
        }
    }

    pub async fn import_anime_batch(&self, titles: Vec<String>) -> AppResult<ImportResult> {
        let mut imported = Vec::new();
        let mut failed = Vec::new();
        let mut skipped = Vec::new();

        const BATCH_SIZE: usize = 3;
        const DELAY_BETWEEN_BATCHES: Duration = Duration::from_millis(1000);

        // Process in batches to respect rate limits
        for chunk in titles.chunks(BATCH_SIZE) {
            let mut batch_futures = Vec::new();

            for title in chunk {
                let title_clone = title.clone();
                let jikan_client = Arc::clone(&self.jikan_client);

                batch_futures.push(async move {
                    (
                        title_clone.clone(),
                        jikan_client.search_anime(&title_clone, 1).await,
                    )
                });
            }

            // Execute batch concurrently
            let results = futures::future::join_all(batch_futures).await;

            for (title, result) in results {
                match result {
                    Ok(anime_list) if !anime_list.is_empty() => {
                        let anime = anime_list.into_iter().next().unwrap();

                        // Check if already exists by mal_id
                        if let Ok(Some(existing)) =
                            self.anime_repo.find_by_mal_id(anime.mal_id).await
                        {
                            skipped.push(SkippedAnime {
                                title: existing.title.clone(),
                                mal_id: anime.mal_id,
                                reason: "Already in database".to_string(),
                            });
                            continue;
                        }

                        // Try to save
                        match self.anime_repo.save(&anime).await {
                            Ok(saved_anime) => {
                                imported.push(ImportedAnime {
                                    title: saved_anime.title.clone(),
                                    mal_id: saved_anime.mal_id,
                                    id: saved_anime.id,
                                });
                            }
                            Err(e) => {
                                // Check if it's a duplicate error
                                if e.to_string().contains("duplicate") {
                                    skipped.push(SkippedAnime {
                                        title: anime.title.clone(),
                                        mal_id: anime.mal_id,
                                        reason: "Duplicate entry".to_string(),
                                    });
                                } else {
                                    failed.push(ImportError {
                                        title: title.clone(),
                                        reason: format!("Database error: {}", e),
                                    });
                                }
                            }
                        }
                    }
                    Ok(_) => {
                        failed.push(ImportError {
                            title,
                            reason: "No results found on MyAnimeList".to_string(),
                        });
                    }
                    Err(e) => {
                        failed.push(ImportError {
                            title,
                            reason: format!("Search failed: {}", e),
                        });
                    }
                }
            }

            // Delay between batches to respect rate limits
            if chunk.len() == BATCH_SIZE && !titles.is_empty() {
                sleep(DELAY_BETWEEN_BATCHES).await;
            }
        }

        Ok(ImportResult {
            imported,
            failed,
            skipped,
            total: u32::try_from(titles.len()).unwrap_or(u32::MAX),
        })
    }

    pub async fn import_from_mal_ids(&self, mal_ids: Vec<i32>) -> AppResult<ImportResult> {
        let mut imported = Vec::new();
        let mut failed = Vec::new();
        let mut skipped = Vec::new();

        const DELAY_BETWEEN_REQUESTS: Duration = Duration::from_millis(1000);

        for mal_id in &mal_ids {
            // Check if already exists
            if let Ok(Some(existing)) = self.anime_repo.find_by_mal_id(*mal_id).await {
                skipped.push(SkippedAnime {
                    title: existing.title.clone(),
                    mal_id: *mal_id,
                    reason: "Already in database".to_string(),
                });
                continue;
            }

            // Fetch from Jikan
            match self.jikan_client.get_anime_by_id(*mal_id).await {
                Ok(Some(anime)) => {
                    // Try to save
                    match self.anime_repo.save(&anime).await {
                        Ok(saved_anime) => {
                            imported.push(ImportedAnime {
                                title: saved_anime.title.clone(),
                                mal_id: saved_anime.mal_id,
                                id: saved_anime.id,
                            });
                        }
                        Err(e) => {
                            if e.to_string().contains("duplicate") {
                                skipped.push(SkippedAnime {
                                    title: anime.title.clone(),
                                    mal_id: *mal_id,
                                    reason: "Duplicate entry".to_string(),
                                });
                            } else {
                                failed.push(ImportError {
                                    title: format!("MAL ID: {}", mal_id),
                                    reason: format!("Database error: {}", e),
                                });
                            }
                        }
                    }
                }
                Ok(None) => {
                    failed.push(ImportError {
                        title: format!("MAL ID: {}", mal_id),
                        reason: "Anime not found on MyAnimeList".to_string(),
                    });
                }
                Err(e) => {
                    failed.push(ImportError {
                        title: format!("MAL ID: {}", mal_id),
                        reason: format!("API error: {}", e),
                    });
                }
            }

            // Rate limiting
            sleep(DELAY_BETWEEN_REQUESTS).await;
        }

        Ok(ImportResult {
            imported,
            failed,
            skipped,
            total: u32::try_from(mal_ids.len()).unwrap_or(u32::MAX),
        })
    }

    pub async fn import_from_csv(&self, csv_content: &str) -> AppResult<ImportResult> {
        let mut reader = csv::Reader::from_reader(csv_content.as_bytes());
        let mut titles = Vec::new();
        let mut mal_ids = Vec::new();

        // Try to detect if CSV contains MAL IDs or titles
        for result in reader.records() {
            match result {
                Ok(record) => {
                    // Check if first column is a number (MAL ID) or text (title)
                    if let Some(first_field) = record.get(0) {
                        if let Ok(mal_id) = first_field.parse::<i32>() {
                            mal_ids.push(mal_id);
                        } else if !first_field.trim().is_empty() {
                            titles.push(first_field.trim().to_string());
                        }
                    }
                }
                Err(e) => {
                    return Err(AppError::InvalidInput(format!("Invalid CSV: {}", e)));
                }
            }
        }

        // Import by MAL IDs if we found any, otherwise by titles
        if !mal_ids.is_empty() {
            self.import_from_mal_ids(mal_ids).await
        } else if !titles.is_empty() {
            self.import_anime_batch(titles).await
        } else {
            Err(AppError::InvalidInput(
                "No valid data found in CSV".to_string(),
            ))
        }
    }

    /// Phase 1: Smart validation with duplicate detection across title variations
    pub async fn validate_anime_titles(&self, titles: Vec<String>) -> AppResult<ValidationResult> {
        let mut found = Vec::new();
        let mut not_found = Vec::new();
        let mut already_exists = Vec::new();
        let mut needs_api_check = Vec::new();

        // Phase 1: Check local database first for all title variations
        for title in titles.iter() {
            match self.anime_repo.find_by_title_variations(title).await {
                Ok(Some(existing_anime)) => {
                    // Determine which field matched
                    let matched_field = if existing_anime
                        .title
                        .to_lowercase()
                        .replace(" ", "")
                        .replace(":", "")
                        .replace("-", "")
                        == title
                            .to_lowercase()
                            .replace(" ", "")
                            .replace(":", "")
                            .replace("-", "")
                    {
                        "title"
                    } else if existing_anime.title_english.as_ref().map(|t| {
                        t.to_lowercase()
                            .replace(" ", "")
                            .replace(":", "")
                            .replace("-", "")
                    }) == Some(
                        title
                            .to_lowercase()
                            .replace(" ", "")
                            .replace(":", "")
                            .replace("-", ""),
                    ) {
                        "title_english"
                    } else if existing_anime.title_japanese.as_ref().map(|t| {
                        t.to_lowercase()
                            .replace(" ", "")
                            .replace(":", "")
                            .replace("-", "")
                    }) == Some(
                        title
                            .to_lowercase()
                            .replace(" ", "")
                            .replace(":", "")
                            .replace("-", ""),
                    ) {
                        "title_japanese"
                    } else {
                        "fuzzy_match"
                    };

                    already_exists.push(ExistingAnime {
                        input_title: title.clone(),
                        matched_title: existing_anime.title.clone(),
                        matched_field: matched_field.to_string(),
                        anime: existing_anime,
                    });
                }
                Ok(None) => {
                    needs_api_check.push(title.clone());
                }
                Err(e) => {
                    not_found.push(ImportError {
                        title: title.clone(),
                        reason: format!("Database error: {}", e),
                    });
                }
            }
        }

        // Phase 2: API calls only for titles not in database
        const BATCH_SIZE: usize = 3;
        const DELAY_BETWEEN_BATCHES: Duration = Duration::from_millis(1000);

        for chunk in needs_api_check.chunks(BATCH_SIZE) {
            let mut batch_futures = Vec::new();

            for title in chunk {
                let title_clone = title.clone();
                let jikan_client = Arc::clone(&self.jikan_client);
                batch_futures.push(async move {
                    (
                        title_clone.clone(),
                        jikan_client.search_anime(&title_clone, 1).await,
                    )
                });
            }

            let results = futures::future::join_all(batch_futures).await;

            for (title, result) in results {
                match result {
                    Ok(anime_list) if !anime_list.is_empty() => {
                        let anime = anime_list.into_iter().next().unwrap();

                        // Double-check: Maybe API returned anime we have under different title
                        match self.anime_repo.find_by_mal_id(anime.mal_id).await {
                            Ok(Some(existing)) => {
                                already_exists.push(ExistingAnime {
                                    input_title: title,
                                    matched_title: existing.title.clone(),
                                    matched_field: "mal_id".to_string(),
                                    anime: existing,
                                });
                            }
                            _ => {
                                found.push(ValidatedAnime {
                                    input_title: title,
                                    anime_data: anime,
                                });
                            }
                        }
                    }
                    Ok(_) => {
                        not_found.push(ImportError {
                            title,
                            reason: "No results found on MyAnimeList".to_string(),
                        });
                    }
                    Err(e) => {
                        not_found.push(ImportError {
                            title,
                            reason: format!("Search failed: {}", e),
                        });
                    }
                }
            }

            // Rate limiting between batches
            if chunk.len() == BATCH_SIZE && !needs_api_check.is_empty() {
                sleep(DELAY_BETWEEN_BATCHES).await;
            }
        }

        Ok(ValidationResult {
            found,
            not_found,
            already_exists,
            total: u32::try_from(titles.len()).unwrap_or(u32::MAX),
        })
    }

    /// Phase 2: Import only selected anime (no API calls, just save to DB)
    pub async fn import_validated_anime(
        &self,
        validated_anime: Vec<ValidatedAnime>,
    ) -> AppResult<ImportResult> {
        let mut imported = Vec::new();
        let mut failed = Vec::new();
        let skipped = Vec::new(); // No skips in this phase since we already validated

        let total_count = validated_anime.len();

        for validated in &validated_anime {
            match self.anime_repo.save(&validated.anime_data).await {
                Ok(saved_anime) => {
                    imported.push(ImportedAnime {
                        title: saved_anime.title.clone(),
                        mal_id: saved_anime.mal_id,
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
            total: u32::try_from(total_count).unwrap_or(u32::MAX),
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
    pub mal_id: i32,
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
    pub mal_id: i32,
    pub reason: String,
}

// New validation types for two-phase import
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
    pub anime_data: crate::domain::entities::Anime,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ExistingAnime {
    pub input_title: String,
    pub matched_title: String,
    pub matched_field: String, // "title", "title_english", "title_japanese", or "mal_id"
    pub anime: crate::domain::entities::Anime,
}
