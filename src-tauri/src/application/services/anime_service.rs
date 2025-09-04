use crate::domain::{entities::Anime, repositories::AnimeRepository, services::ScoreCalculator};
use crate::infrastructure::{cache::RedisCache, external::jikan::JikanClient};
use crate::shared::errors::AppResult;
use serde_json;
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

    pub async fn get_anime_by_mal_id(&self, mal_id: i32) -> AppResult<Option<Anime>> {
        // Check cache
        let cache_key = format!("anime:mal:{}", mal_id);
        if let Ok(Some(cached)) = self.cache.get::<Anime>(&cache_key).await {
            return Ok(Some(cached));
        }

        // Check database
        if let Some(anime) = self.anime_repo.find_by_mal_id(mal_id).await? {
            let _ = self.cache.set(&cache_key, &anime, 7200).await;
            return Ok(Some(anime));
        }

        // Fetch from Jikan
        if let Some(anime) = self.jikan_client.get_anime_by_id(mal_id).await? {
            // Save to database
            let saved_anime = self.anime_repo.save(&anime).await?;

            // Cache the result
            let _ = self.cache.set(&cache_key, &saved_anime, 7200).await;

            return Ok(Some(saved_anime));
        }

        Ok(None)
    }

    pub async fn update_anime(&self, anime: &Anime) -> AppResult<Anime> {
        // Update in database
        let updated = self.anime_repo.update(anime).await?;

        // Invalidate cache
        let cache_key = format!("anime:{}", anime.id);
        let _ = self.cache.delete(&cache_key).await;

        // Also invalidate MAL ID cache if present
        if let Some(mal_id) = anime.mal_id {
            let mal_cache_key = format!("anime:mal:{}", mal_id);
            let _ = self.cache.delete(&mal_cache_key).await;
        }

        Ok(updated)
    }

    pub async fn delete_anime(&self, id: &Uuid) -> AppResult<()> {
        // Delete from database
        self.anime_repo.delete(id).await?;

        // Invalidate cache
        let cache_key = format!("anime:{}", id);
        let _ = self.cache.delete(&cache_key).await;

        Ok(())
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

    pub async fn recalculate_scores(&self, id: &Uuid) -> AppResult<Anime> {
        // Get anime from database
        let mut anime = self.anime_repo.find_by_id(id).await?.ok_or_else(|| {
            crate::shared::errors::AppError::NotFound(format!("Anime with ID {} not found", id))
        })?;

        // Recalculate scores
        anime.update_scores(&self.score_calculator);

        // Update in database
        let updated = self.anime_repo.update(&anime).await?;

        // Invalidate cache
        let cache_key = format!("anime:{}", id);
        let _ = self.cache.delete(&cache_key).await;

        Ok(updated)
    }

    pub async fn get_recommendations(&self, anime_id: &Uuid) -> AppResult<Vec<Anime>> {
        // Get the anime
        let anime = self.anime_repo.find_by_id(anime_id).await?.ok_or_else(|| {
            crate::shared::errors::AppError::NotFound(format!(
                "Anime with ID {} not found",
                anime_id
            ))
        })?;

        // If we have MAL ID, get recommendations from Jikan
        if let Some(mal_id) = anime.mal_id {
            let recommendations = self.jikan_client.get_anime_recommendations(mal_id).await?;

            if !recommendations.is_empty() {
                // Save to database
                let _ = self.anime_repo.save_batch(&recommendations).await;
                return Ok(recommendations);
            }
        }

        // Fallback: Get anime with similar genres from database
        // This is a simplified recommendation system
        Ok(Vec::new())
    }
}
