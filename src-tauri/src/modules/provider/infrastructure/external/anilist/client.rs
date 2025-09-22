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
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

use super::{
    dto::{AniListRequest, AniListResponse, MediaResponse, PageResponse},
    graphql::AniListQueries,
    mapper::AniListMapper,
};

pub struct AniListClient {
    client: Client,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
    cached_behavior: CachedProviderBehavior,
}

impl AniListClient {
    pub fn new() -> AppResult<Self> {
        Self::with_cache(Arc::new(ProviderCache::default()))
    }

    pub fn with_cache(cache: Arc<ProviderCache>) -> AppResult<Self> {
        let client = CommonHttpHandler::create_http_client(30, "Miru-Anime-App/1.0")?;

        Ok(Self {
            client,
            base_url: "https://graphql.anilist.co".to_string(),
            // AniList current rate limit: 30 requests per minute = 0.5 per second
            rate_limiter: Arc::new(RateLimiter::new(0.5)),
            cached_behavior: CachedProviderBehavior::new(cache, AnimeProvider::AniList),
        })
    }

    /// Search for anime using GraphQL - this is AniList's main strength
    /// Use this when Jikan search fails or returns no results
    pub async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        let limit = limit.min(50); // AniList max limit is 50
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

        // Use organized GraphQL queries
        let graphql_query = AniListQueries::search_anime();
        let variables = AniListQueries::search_variables(query, limit);

        let response = self.execute_query(graphql_query, Some(variables)).await?;
        let page_response: PageResponse = serde_json::from_value(response).map_err(|e| {
            AppError::ApiError(format!("Failed to parse AniList search response: {}", e))
        })?;

        Ok(page_response
            .page
            .media
            .into_iter()
            .map(AniListMapper::to_domain)
            .collect())
    }

    /// Get anime by AniList ID
    pub async fn get_anime_by_anilist_id(
        &self,
        anilist_id: i32,
    ) -> AppResult<Option<AnimeDetailed>> {
        self.rate_limiter.wait().await?;

        // Use organized GraphQL queries
        let graphql_query = AniListQueries::get_by_id();
        let variables = AniListQueries::get_by_id_variables(anilist_id);

        let response = self.execute_query(graphql_query, Some(variables)).await?;
        let media_response: MediaResponse = serde_json::from_value(response).map_err(|e| {
            AppError::ApiError(format!("Failed to parse AniList get_by_id response: {}", e))
        })?;

        Ok(media_response.media.map(AniListMapper::to_domain))
    }

    /// Get top/trending anime from AniList
    pub async fn get_top_anime(&self, page: i32, limit: i32) -> AppResult<Vec<AnimeDetailed>> {
        self.rate_limiter.wait().await?;

        let graphql_query = AniListQueries::trending_anime();
        let variables = AniListQueries::trending_variables(limit as usize, page);

        let response = self.execute_query(graphql_query, Some(variables)).await?;
        let page_response: PageResponse = serde_json::from_value(response).map_err(|e| {
            AppError::ApiError(format!("Failed to parse AniList trending response: {}", e))
        })?;

        Ok(page_response
            .page
            .media
            .into_iter()
            .map(AniListMapper::to_domain)
            .collect())
    }

    /// Get seasonal anime from AniList
    pub async fn get_seasonal_anime(
        &self,
        year: i32,
        season: &str,
        page: i32,
    ) -> AppResult<Vec<AnimeDetailed>> {
        // Validate season format
        let season_normalized = match season.to_lowercase().as_str() {
            "spring" => "SPRING",
            "summer" => "SUMMER",
            "fall" | "autumn" => "FALL",
            "winter" => "WINTER",
            _ => {
                return Err(AppError::ValidationError(format!(
                    "Invalid season '{}'. Must be one of: spring, summer, fall, winter",
                    season
                )))
            }
        };

        self.rate_limiter.wait().await?;

        let graphql_query = AniListQueries::seasonal_anime();
        let variables = AniListQueries::seasonal_variables(year, season_normalized, page);

        let response = self.execute_query(graphql_query, Some(variables)).await?;
        let page_response: PageResponse = serde_json::from_value(response).map_err(|e| {
            AppError::ApiError(format!("Failed to parse AniList seasonal response: {}", e))
        })?;

        Ok(page_response
            .page
            .media
            .into_iter()
            .map(AniListMapper::to_domain)
            .collect())
    }

    /// Execute a GraphQL query with retry logic
    async fn execute_query(&self, query: &str, variables: Option<Value>) -> AppResult<Value> {
        let request = AniListRequest {
            query: query.to_string(),
            variables,
        };

        let response = CommonHttpHandler::execute_with_retry(
            || self.client.post(&self.base_url).json(&request).send(),
            "AniList",
            "GraphQL query",
        )
        .await?;

        let anilist_response: AniListResponse<Value> = response
            .json()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to parse AniList response: {}", e)))?;

        // Handle GraphQL errors
        if let Some(errors) = anilist_response.errors {
            let error_messages: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            return Err(AppError::ApiError(format!(
                "AniList GraphQL errors: {}",
                error_messages.join(", ")
            )));
        }

        anilist_response
            .data
            .ok_or_else(|| AppError::ApiError("AniList response contained no data".to_string()))
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
        info!("AniList client cache cleared");
    }
}

#[async_trait]
impl AnimeProviderClient for AniListClient {
    fn provider_type(&self) -> AnimeProvider {
        AnimeProvider::AniList
    }

    fn get_rate_limit_info(&self) -> RateLimiterInfo {
        self.rate_limiter.get_info()
    }

    async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        self.search_anime(query, limit).await
    }

    async fn get_anime_by_id(&self, id: &str) -> AppResult<Option<AnimeDetailed>> {
        // Parse ID as AniList ID (integer)
        let anilist_id = id
            .parse::<i32>()
            .map_err(|_| AppError::InvalidInput("AniList requires numeric ID".to_string()))?;

        self.get_anime_by_anilist_id(anilist_id).await
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
