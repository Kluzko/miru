use crate::domain::repositories::AnimeRepository;
use crate::infrastructure::external::jikan::JikanClient;
use crate::shared::errors::{AppError, AppResult};
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
        let batch_size = 3;
        let delay_between_batches = Duration::from_millis(1000);

        // Process in batches to respect rate limits
        for chunk in titles.chunks(batch_size) {
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
                        // Take the first result
                        let anime = anime_list.into_iter().next().unwrap();

                        // Check if already exists
                        if let Some(mal_id) = anime.mal_id {
                            if let Ok(Some(_)) = self.anime_repo.find_by_mal_id(mal_id).await {
                                failed.push(ImportError {
                                    title: title.clone(),
                                    reason: "Already exists in database".to_string(),
                                });
                                continue;
                            }
                        }

                        // Save to database
                        match self.anime_repo.save(&anime).await {
                            Ok(saved_anime) => {
                                imported.push(ImportedAnime {
                                    title: saved_anime.title.clone(),
                                    mal_id: saved_anime.mal_id,
                                    id: saved_anime.id,
                                });
                            }
                            Err(e) => {
                                failed.push(ImportError {
                                    title,
                                    reason: format!("Failed to save: {}", e),
                                });
                            }
                        }
                    }
                    Ok(_) => {
                        failed.push(ImportError {
                            title,
                            reason: "No results found".to_string(),
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
            if chunk.len() == batch_size {
                sleep(delay_between_batches).await;
            }
        }

        Ok(ImportResult {
            imported,
            failed,
            total: titles.len(),
        })
    }

    pub async fn import_from_csv(&self, csv_content: &str) -> AppResult<ImportResult> {
        let mut reader = csv::Reader::from_reader(csv_content.as_bytes());
        let mut titles = Vec::new();

        for result in reader.records() {
            match result {
                Ok(record) => {
                    // Assume first column is title
                    if let Some(title) = record.get(0) {
                        titles.push(title.to_string());
                    }
                }
                Err(e) => {
                    return Err(AppError::InvalidInput(format!("Invalid CSV: {}", e)));
                }
            }
        }

        if titles.is_empty() {
            return Err(AppError::InvalidInput("No titles found in CSV".to_string()));
        }

        self.import_anime_batch(titles).await
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportResult {
    pub imported: Vec<ImportedAnime>,
    pub failed: Vec<ImportError>,
    pub total: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportedAnime {
    pub title: String,
    pub mal_id: Option<i32>,
    pub id: uuid::Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportError {
    pub title: String,
    pub reason: String,
}
