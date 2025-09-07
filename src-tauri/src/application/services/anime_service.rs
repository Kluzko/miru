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
        let db_results = self.anime_repo.search(query, 20).await?;

        // If we have results, return them
        // The user can explicitly "search online" if they want fresh data
        if !db_results.is_empty() {
            return Ok(db_results);
        }

        // No local results? Search Jikan and save
        let jikan_results = self.jikan_client.search_anime(query, 10).await?;

        if !jikan_results.is_empty() {
            // Save to database for next time
            let _ = self.anime_repo.save_batch(&jikan_results).await;
        }

        Ok(jikan_results)
    }

    pub async fn get_anime_by_id(&self, id: &Uuid) -> AppResult<Option<Anime>> {
        // Get from database
        let anime = self.anime_repo.find_by_id(id).await?;

        Ok(anime)
    }

    pub async fn get_top_anime(&self, page: i32, limit: i32) -> AppResult<Vec<Anime>> {
        // Fetch from Jikan
        let anime_list = self.jikan_client.get_top_anime(page, limit).await?;

        // Save to database
        if !anime_list.is_empty() {
            let _ = self.anime_repo.save_batch(&anime_list).await;
        }

        Ok(anime_list)
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

        // Save to database
        if !anime_list.is_empty() {
            let _ = self.anime_repo.save_batch(&anime_list).await;
        }

        Ok(anime_list)
    }
}
