use crate::domain::{entities::Anime, repositories::AnimeRepository, services::ScoreCalculator};
use crate::infrastructure::external::jikan::JikanClient;
use crate::shared::errors::AppResult;
use std::sync::Arc;
use uuid::Uuid;

pub struct AnimeService {
    anime_repo: Arc<dyn AnimeRepository>,
    jikan_client: Arc<JikanClient>,
    score_calculator: Arc<ScoreCalculator>,
}

impl AnimeService {
    pub fn new(anime_repo: Arc<dyn AnimeRepository>, jikan_client: Arc<JikanClient>) -> Self {
        Self {
            anime_repo,
            jikan_client,
            score_calculator: Arc::new(ScoreCalculator::new()),
        }
    }

    pub async fn search_anime(&self, query: &str) -> AppResult<Vec<Anime>> {
        // First, search in local database
        let db_results = self.anime_repo.search(query, 20).await?;

        // If we have sufficient results, return them
        if db_results.len() >= 5 {
            return Ok(db_results);
        }

        // Otherwise, search Jikan and merge results
        let jikan_results = self.jikan_client.search_anime(query, 10).await?;

        if !jikan_results.is_empty() {
            // Save new anime to database (the repository will handle duplicates)
            for anime in &jikan_results {
                // The save method will handle duplicates by mal_id
                let _ = self.anime_repo.save(anime).await;
            }

            // Search again to get all results (including newly saved ones)
            return self.anime_repo.search(query, 20).await;
        }

        Ok(db_results)
    }

    pub async fn get_anime_by_id(&self, id: &Uuid) -> AppResult<Option<Anime>> {
        self.anime_repo.find_by_id(id).await
    }

    pub async fn get_anime_by_mal_id(&self, mal_id: i32) -> AppResult<Option<Anime>> {
        // First check local database
        if let Some(anime) = self.anime_repo.find_by_mal_id(mal_id).await? {
            return Ok(Some(anime));
        }

        // If not found locally, fetch from Jikan
        if let Some(anime) = self.jikan_client.get_anime_by_id(mal_id).await? {
            // Save to database
            let saved = self.anime_repo.save(&anime).await?;
            return Ok(Some(saved));
        }

        Ok(None)
    }

    pub async fn get_top_anime(&self, page: i32, limit: i32) -> AppResult<Vec<Anime>> {
        // Always fetch fresh data from Jikan for top anime
        let anime_list = self.jikan_client.get_top_anime(page, limit).await?;

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
        page: i32,
    ) -> AppResult<Vec<Anime>> {
        // Fetch from Jikan
        let anime_list = self
            .jikan_client
            .get_seasonal_anime(year, season, page)
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

    pub async fn update_anime(&self, anime: &Anime) -> AppResult<Anime> {
        // Recalculate scores before saving
        let mut updated_anime = anime.clone();
        updated_anime.update_scores(&self.score_calculator);

        self.anime_repo.update(&updated_anime).await
    }

    pub async fn delete_anime(&self, id: &Uuid) -> AppResult<()> {
        self.anime_repo.delete(id).await
    }

    pub async fn refresh_anime_from_mal(&self, mal_id: i32) -> AppResult<Option<Anime>> {
        // Fetch fresh data from Jikan
        if let Some(anime) = self.jikan_client.get_anime_by_id(mal_id).await? {
            // Save/update in database
            let saved = self.anime_repo.save(&anime).await?;
            return Ok(Some(saved));
        }

        Ok(None)
    }
}
