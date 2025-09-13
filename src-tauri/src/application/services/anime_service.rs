use super::provider_manager::ProviderManager;
use crate::domain::{
    entities::AnimeDetailed, repositories::AnimeRepository, services::ScoreCalculator,
};
use crate::shared::errors::AppResult;
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
        // First, search in local database
        let db_results = self.anime_repo.search(query, 20).await?;

        // If we have sufficient results, return them
        if db_results.len() >= 5 {
            return Ok(db_results);
        }

        // Otherwise, search via provider manager and merge results
        let mut provider_manager = self.provider_manager.lock().await;
        let provider_results = provider_manager.search_anime(query, 10).await?;

        if !provider_results.is_empty() {
            // Save new anime to database (the repository will handle duplicates)
            for anime in &provider_results {
                let _ = self.anime_repo.save(anime).await;
            }

            // Search again to get all results (including newly saved ones)
            return self.anime_repo.search(query, 20).await;
        }

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
                    eprintln!("Warning: Failed to save anime {}: {}", anime.title, e);
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
                    eprintln!("Warning: Failed to save anime {}: {}", anime.title, e);
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
