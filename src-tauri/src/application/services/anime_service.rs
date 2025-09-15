use super::provider_manager::ProviderManager;
use crate::domain::{
    entities::AnimeDetailed, repositories::AnimeRepository, services::ScoreCalculator,
};
use crate::shared::errors::AppResult;
use crate::shared::utils::logger::LogContext;
use crate::{log_debug, log_info};
use std::sync::Arc;
use uuid::Uuid;

pub struct AnimeService {
    anime_repo: Arc<dyn AnimeRepository>,
    provider_manager: Arc<tokio::sync::Mutex<ProviderManager>>,
    #[allow(dead_code)]
    score_calculator: Arc<ScoreCalculator>,
}

impl AnimeService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        provider_manager: Arc<tokio::sync::Mutex<ProviderManager>>,
    ) -> Self {
        Self {
            anime_repo,
            provider_manager,
            score_calculator: Arc::new(ScoreCalculator::new()),
        }
    }

    pub async fn search_anime(&self, query: &str) -> AppResult<Vec<AnimeDetailed>> {
        // First, search in local database with improved search
        let db_results = self.anime_repo.search(query, 20).await?;

        // Check for quality matches - look for exact or very close matches
        let has_good_matches = db_results.iter().any(|anime| {
            let query_lower = query.to_lowercase();
            let title_main = anime.title.main.to_lowercase();
            let title_english = anime.title.english.as_ref().map(|t| t.to_lowercase());
            let title_japanese = anime.title.japanese.as_ref().map(|t| t.to_lowercase());

            // Check for exact matches or titles starting with query
            title_main == query_lower
                || title_main.starts_with(&query_lower)
                || title_english
                    .as_ref()
                    .map_or(false, |t| *t == query_lower || t.starts_with(&query_lower))
                || title_japanese
                    .as_ref()
                    .map_or(false, |t| *t == query_lower || t.starts_with(&query_lower))
        });

        // If we have good quality matches OR sufficient quantity, return database results
        if has_good_matches || db_results.len() >= 3 {
            return Ok(db_results);
        }

        // Otherwise, search via provider manager for potentially better results
        let mut provider_manager = self.provider_manager.lock().await;
        let provider_results = provider_manager.search_anime(query, 10).await?;

        if !provider_results.is_empty() {
            log_debug!(
                "Database search yielded {} results, trying external APIs for '{}'",
                db_results.len(),
                query
            );

            // Save new anime to database (the repository will handle duplicates)
            for anime in &provider_results {
                match self.anime_repo.save(anime).await {
                    Ok(_) => log_info!(
                        "Successfully saved anime from external API: {}",
                        anime.title.main
                    ),
                    Err(e) => LogContext::error_with_context(
                        &e,
                        &format!("Failed to save anime {}", anime.title.main),
                    ),
                }
            }

            // Search again to get all results (including newly saved ones)
            let combined_results = self.anime_repo.search(query, 20).await?;
            log_info!(
                "After external API search and save, found {} total results",
                combined_results.len()
            );
            return Ok(combined_results);
        }

        // If external APIs also failed, return what we found in the database
        Ok(db_results)
    }

    pub async fn get_anime_by_id(&self, id: &Uuid) -> AppResult<Option<AnimeDetailed>> {
        self.anime_repo.find_by_id(id).await
    }

    pub async fn get_top_anime(&self, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        // Always fetch fresh data via provider manager for top anime
        let mut provider_manager = self.provider_manager.lock().await;
        let anime_list = provider_manager.get_top_anime(limit).await?;

        // Save to database (handling duplicates)
        let mut saved_anime = Vec::new();
        for anime in anime_list {
            match self.anime_repo.save(&anime).await {
                Ok(saved) => saved_anime.push(saved),
                Err(e) => {
                    // Log but don't fail the entire operation
                    LogContext::error_with_context(
                        &e,
                        &format!("Failed to save anime {}", anime.title),
                    );
                    // Still include the anime in results even if save failed
                    saved_anime.push(anime);
                }
            }
        }

        Ok(saved_anime)
    }

    pub async fn get_seasonal_anime(
        &self,
        year: i32,
        season: &str,
        limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        // Fetch via provider manager
        let mut provider_manager = self.provider_manager.lock().await;
        let anime_list = provider_manager
            .get_seasonal_anime(year, season, limit)
            .await?;

        // Save to database (handling duplicates)
        let mut saved_anime = Vec::new();
        for anime in anime_list {
            match self.anime_repo.save(&anime).await {
                Ok(saved) => saved_anime.push(saved),
                Err(e) => {
                    // Log but don't fail the entire operation
                    LogContext::error_with_context(
                        &e,
                        &format!("Failed to save anime {}", anime.title),
                    );
                    // Still include the anime in results even if save failed
                    saved_anime.push(anime);
                }
            }
        }

        Ok(saved_anime)
    }

    #[allow(dead_code)]
    pub async fn update_anime(&self, anime: &AnimeDetailed) -> AppResult<AnimeDetailed> {
        // Recalculate scores before saving
        let mut updated_anime = anime.clone();
        updated_anime.update_scores(&self.score_calculator);

        self.anime_repo.update(&updated_anime).await
    }

    #[allow(dead_code)]
    pub async fn delete_anime(&self, id: &Uuid) -> AppResult<()> {
        self.anime_repo.delete(id).await
    }
}
