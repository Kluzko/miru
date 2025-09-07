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
                        if let Some(mal_id) = anime.mal_id {
                            if let Ok(Some(existing)) = self.anime_repo.find_by_mal_id(mal_id).await
                            {
                                skipped.push(SkippedAnime {
                                    title: existing.title.clone(),
                                    mal_id: Some(mal_id),
                                    reason: "Already in database".to_string(),
                                });
                                continue;
                            }
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
                    mal_id: Some(*mal_id),
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
                                    mal_id: Some(*mal_id),
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
    pub mal_id: Option<i32>,
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
    pub mal_id: Option<i32>,
    pub reason: String,
}
