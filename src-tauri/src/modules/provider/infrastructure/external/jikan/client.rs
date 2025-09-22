use crate::modules::anime::AnimeDetailed;
use crate::modules::provider::infrastructure::external::common::CachedProviderBehavior;
use crate::modules::provider::infrastructure::external::CommonHttpHandler;
use crate::modules::provider::traits::{AnimeProviderClient, RateLimiterInfo};
use crate::modules::provider::{AnimeProvider, ProviderCache};
use crate::shared::{
    errors::{AppError, AppResult},
    utils::RateLimiter,
};
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use tracing::info;

use super::{
    dto::{JikanAnimeListResponse, JikanAnimeResponse, JikanSearchParams},
    mapper::JikanMapper,
};

pub struct JikanClient {
    client: Client,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
    cached_behavior: CachedProviderBehavior,
}

impl JikanClient {
    pub fn new() -> AppResult<Self> {
        Self::with_cache(Arc::new(ProviderCache::default()))
    }

    pub fn with_cache(cache: Arc<ProviderCache>) -> AppResult<Self> {
        let client = CommonHttpHandler::create_http_client(30, "Miru-Anime-App/1.0")?;

        Ok(Self {
            client,
            base_url: "https://api.jikan.moe/v4".to_string(),
            rate_limiter: Arc::new(RateLimiter::new(3.0)), // 3 requests per second for Jikan (official limit)
            cached_behavior: CachedProviderBehavior::new(cache, AnimeProvider::Jikan),
        })
    }

    pub async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        let limit = limit.min(25); // Jikan max limit is 25
        self.cached_behavior
            .cached_search(query, limit, || self.search_anime_uncached(query, limit))
            .await
    }

    async fn search_anime_uncached(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        self.rate_limiter.wait().await?;

        let params = JikanSearchParams {
            q: Some(query.trim().to_string()),
            limit: Some(limit as i32),
            sfw: Some(true),
            ..Default::default()
        };

        let url = format!("{}/anime", self.base_url);
        let response = CommonHttpHandler::execute_with_retry(
            || self.client.get(&url).query(&params).send(),
            "Jikan",
            "search anime",
        )
        .await?;

        let jikan_response = response
            .json::<JikanAnimeListResponse>()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to parse Jikan response: {}", e)))?;

        Ok(jikan_response
            .data
            .into_iter()
            .map(JikanMapper::to_domain)
            .collect())
    }

    pub async fn get_anime_by_id(&self, mal_id: i32) -> AppResult<Option<AnimeDetailed>> {
        self.rate_limiter.wait().await?;

        let url = format!("{}/anime/{}", self.base_url, mal_id);
        let response = match CommonHttpHandler::execute_with_retry(
            || self.client.get(&url).send(),
            "Jikan",
            "get anime by ID",
        )
        .await
        {
            Ok(response) => response,
            Err(AppError::NotFound(_)) => return Ok(None),
            Err(e) => return Err(e),
        };

        let jikan_response = response
            .json::<JikanAnimeResponse>()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to parse Jikan response: {}", e)))?;

        Ok(Some(JikanMapper::to_domain(jikan_response.data)))
    }

    pub async fn get_top_anime(&self, page: i32, limit: i32) -> AppResult<Vec<AnimeDetailed>> {
        self.rate_limiter.wait().await?;

        let url = format!("{}/top/anime", self.base_url);
        let response = CommonHttpHandler::execute_with_retry(
            || {
                self.client
                    .get(&url)
                    .query(&[("page", page.to_string()), ("limit", limit.to_string())])
                    .send()
            },
            "Jikan",
            "get top anime",
        )
        .await?;

        let jikan_response = response
            .json::<JikanAnimeListResponse>()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to parse Jikan response: {}", e)))?;

        Ok(jikan_response
            .data
            .into_iter()
            .map(JikanMapper::to_domain)
            .collect())
    }

    pub async fn get_seasonal_anime(
        &self,
        year: i32,
        season: &str,
        page: i32,
    ) -> AppResult<Vec<AnimeDetailed>> {
        let season_lower = season.to_lowercase();
        if !["spring", "summer", "fall", "winter"].contains(&season_lower.as_str()) {
            return Err(AppError::ValidationError(format!(
                "Invalid season '{}'. Must be one of: spring, summer, fall, winter",
                season
            )));
        }

        self.rate_limiter.wait().await?;

        let url = format!("{}/seasons/{}/{}", self.base_url, year, season_lower);
        let response = CommonHttpHandler::execute_with_retry(
            || {
                self.client
                    .get(&url)
                    .query(&[("page", page.to_string())])
                    .send()
            },
            "Jikan",
            "get seasonal anime",
        )
        .await?;

        let jikan_response = response
            .json::<JikanAnimeListResponse>()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to parse Jikan response: {}", e)))?;

        Ok(jikan_response
            .data
            .into_iter()
            .map(JikanMapper::to_domain)
            .collect())
    }

    /// Get cache statistics for monitoring
    pub async fn get_cache_stats(
        &self,
    ) -> crate::modules::provider::infrastructure::cache::provider_cache::CacheStats {
        self.cached_behavior.get_cache_stats().await
    }

    /// Clear all cached data
    pub async fn clear_cache(&self) {
        self.cached_behavior.clear_cache().await;
        info!("Jikan client cache cleared");
    }
}

#[async_trait]
impl AnimeProviderClient for JikanClient {
    fn provider_type(&self) -> AnimeProvider {
        AnimeProvider::Jikan
    }

    fn get_rate_limit_info(&self) -> RateLimiterInfo {
        self.rate_limiter.get_info()
    }

    async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        self.search_anime(query, limit).await
    }

    async fn get_anime_by_id(&self, id: &str) -> AppResult<Option<AnimeDetailed>> {
        let id_num = id
            .parse::<i32>()
            .map_err(|_| AppError::InvalidInput("Jikan requires numeric ID".to_string()))?;
        self.get_anime_by_id(id_num).await
    }

    async fn get_top_anime(&self, page: i32, limit: i32) -> AppResult<Vec<AnimeDetailed>> {
        self.get_top_anime(page, limit).await
    }

    async fn get_seasonal_anime(
        &self,
        year: i32,
        season: &str,
        page: i32,
    ) -> AppResult<Vec<AnimeDetailed>> {
        self.get_seasonal_anime(year, season, page).await
    }
}
