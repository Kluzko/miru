use crate::domain::{
    entities::AnimeDetailed,
    traits::anime_provider_client::{AnimeProviderClient, RateLimiterInfo},
    value_objects::AnimeProvider,
};
use crate::infrastructure::shared::provider_cache::ProviderCache;
use crate::shared::{
    errors::{AppError, AppResult},
    utils::RateLimiter,
};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::{
    dto::{AniListRequest, AniListResponse, PageResponse},
    graphql::AniListQueries,
    mapper::AniListMapper,
};

pub struct AniListClient {
    client: Client,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
    cache: Arc<ProviderCache>,
}

impl AniListClient {
    pub fn new() -> AppResult<Self> {
        Self::with_cache(Arc::new(ProviderCache::default()))
    }

    pub fn with_cache(cache: Arc<ProviderCache>) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Miru-Anime-App/1.0")
            .build()
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self {
            client,
            base_url: "https://graphql.anilist.co".to_string(),
            // AniList current rate limit: 30 requests per minute = 0.5 per second
            rate_limiter: Arc::new(RateLimiter::new(0.5)),
            cache,
        })
    }

    /// Search for anime using GraphQL - this is AniList's main strength
    /// Use this when Jikan search fails or returns no results
    pub async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        if query.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Search query cannot be empty".to_string(),
            ));
        }

        // Create cache key that includes limit to ensure proper caching
        let cache_query = format!("{}:limit:{}", query.trim(), limit);

        // Check cache first
        if let Some(cached_results) = self
            .cache
            .get_search_results(&AnimeProvider::AniList, &cache_query)
            .await
        {
            debug!("AniList search cache hit for query: {}", query);
            return Ok(cached_results);
        }

        // Check if request is already in progress to prevent duplicate concurrent requests
        if self
            .cache
            .is_request_in_progress(&AnimeProvider::AniList, &cache_query)
            .await
        {
            debug!(
                "AniList search already in progress for query: {}, waiting for result",
                query
            );

            // Wait a bit and check cache again
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if let Some(cached_results) = self
                .cache
                .get_search_results(&AnimeProvider::AniList, &cache_query)
                .await
            {
                return Ok(cached_results);
            }

            // If still not available, proceed with the request (fallback)
            warn!(
                "AniList search in progress but no cached result found, proceeding with API call"
            );
        }

        // Mark request as in progress
        self.cache
            .mark_request_in_progress(&AnimeProvider::AniList, &cache_query)
            .await;

        // Perform API call
        let api_result = self.search_anime_uncached(query, limit).await;

        // Remove in-progress marker
        self.cache
            .remove_request_in_progress(&AnimeProvider::AniList, &cache_query)
            .await;

        match api_result {
            Ok(results) => {
                // Cache successful results
                if let Err(e) = self
                    .cache
                    .cache_search_results(&AnimeProvider::AniList, &cache_query, results.clone())
                    .await
                {
                    warn!("Failed to cache AniList search results: {}", e);
                }
                debug!(
                    "AniList search API call completed and cached for query: {}",
                    query
                );
                Ok(results)
            }
            Err(e) => {
                // Don't cache error responses, just return the error
                debug!(
                    "AniList search API call failed for query: {}, error: {}",
                    query, e
                );
                Err(e)
            }
        }
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

    /// Execute a GraphQL query
    async fn execute_query(&self, query: &str, variables: Option<Value>) -> AppResult<Value> {
        let request = AniListRequest {
            query: query.to_string(),
            variables,
        };

        let response = self
            .client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ApiError(format!("AniList request failed: {}", e)))?;

        self.handle_response_status(response.status())?;

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

    fn handle_response_status(&self, status: StatusCode) -> AppResult<()> {
        match status {
            StatusCode::OK => Ok(()),
            StatusCode::TOO_MANY_REQUESTS => Err(AppError::RateLimitError(
                "AniList rate limit exceeded (30 requests/minute)".to_string(),
            )),
            StatusCode::BAD_REQUEST => {
                Err(AppError::ApiError("Bad request to AniList API".to_string()))
            }
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::SERVICE_UNAVAILABLE => Err(
                AppError::ExternalServiceError("AniList service unavailable".to_string()),
            ),
            _ => Err(AppError::ApiError(format!(
                "Unexpected status code from AniList: {}",
                status
            ))),
        }
    }

    /// Get cache statistics for monitoring
    pub async fn get_cache_stats(
        &self,
    ) -> crate::infrastructure::shared::provider_cache::CacheStats {
        self.cache.get_stats().await
    }

    /// Clear all cached data
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
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

    async fn get_anime_by_id(&self, _id: &str) -> AppResult<Option<AnimeDetailed>> {
        // AniList get by ID not implemented yet, return appropriate error
        Err(AppError::NotImplemented(
            "AniList get_by_id not implemented yet".to_string(),
        ))
    }
}
