use crate::domain::{entities::Anime, repositories::AnimeRepository, services::ScoreCalculator};
use crate::infrastructure::{cache::RedisCache, external::jikan::JikanClient};
use crate::shared::errors::AppResult;
use std::sync::Arc;
use uuid::Uuid;

pub struct AnimeService {
    anime_repo: Arc<dyn AnimeRepository>,
    cache: Arc<RedisCache>,
    jikan_client: Arc<JikanClient>,
    score_calculator: Arc<ScoreCalculator>,
}

impl AnimeService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        cache: Arc<RedisCache>,
        jikan_client: Arc<JikanClient>,
    ) -> Self {
        Self {
            anime_repo,
            cache,
            jikan_client,
            score_calculator: Arc::new(ScoreCalculator::new()),
        }
    }

    pub async fn search_anime(&self, query: &str) -> AppResult<Vec<Anime>> {
        // Check cache first
        let cache_key = format!("search:{}", query);
        if let Ok(Some(cached)) = self.cache.get::<Vec<Anime>>(&cache_key).await {
            return Ok(cached);
        }

        // Search in database first
        let db_results = self.anime_repo.search(query, 10).await?;

        if !db_results.is_empty() {
            // Cache the results
            let _ = self.cache.set(&cache_key, &db_results, 3600).await;
            return Ok(db_results);
        }

        // If not in database, fetch from Jikan
        let jikan_results = self.jikan_client.search_anime(query, 10).await?;

        // Save to database for future use
        if !jikan_results.is_empty() {
            let _ = self.anime_repo.save_batch(&jikan_results).await;

            // Cache the results
            let _ = self.cache.set(&cache_key, &jikan_results, 3600).await;
        }

        Ok(jikan_results)
    }

    pub async fn get_anime_by_id(&self, id: &Uuid) -> AppResult<Option<Anime>> {
        // Check cache
        let cache_key = format!("anime:{}", id);
        if let Ok(Some(cached)) = self.cache.get::<Anime>(&cache_key).await {
            return Ok(Some(cached));
        }

        // Get from database
        let anime = self.anime_repo.find_by_id(id).await?;

        if let Some(ref anime_data) = anime {
            // Cache the result
            let _ = self.cache.set(&cache_key, anime_data, 7200).await;
        }

        Ok(anime)
    }

    pub async fn get_top_anime(&self, page: i32, limit: i32) -> AppResult<Vec<Anime>> {
        // Check cache
        let cache_key = format!("top_anime:{}:{}", page, limit);
        if let Ok(Some(cached)) = self.cache.get::<Vec<Anime>>(&cache_key).await {
            return Ok(cached);
        }

        // Fetch from Jikan
        let anime_list = self.jikan_client.get_top_anime(page, limit).await?;

        // Save to database
        if !anime_list.is_empty() {
            let _ = self.anime_repo.save_batch(&anime_list).await;

            // Cache the results
            let _ = self.cache.set(&cache_key, &anime_list, 3600).await;
        }

        Ok(anime_list)
    }

    pub async fn get_seasonal_anime(
        &self,
        year: i32,
        season: &str,
        page: i32,
    ) -> AppResult<Vec<Anime>> {
        // Check cache
        let cache_key = format!("seasonal:{}:{}:{}", year, season, page);
        if let Ok(Some(cached)) = self.cache.get::<Vec<Anime>>(&cache_key).await {
            return Ok(cached);
        }

        // Fetch from Jikan
        let anime_list = self
            .jikan_client
            .get_seasonal_anime(year, season, page)
            .await?;

        // Save to database
        if !anime_list.is_empty() {
            let _ = self.anime_repo.save_batch(&anime_list).await;

            // Cache the results
            let _ = self.cache.set(&cache_key, &anime_list, 3600).await;
        }

        Ok(anime_list)
    }
}
