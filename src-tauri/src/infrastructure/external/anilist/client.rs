use crate::domain::{
    entities::AnimeDetailed,
    traits::anime_provider_client::{AnimeProviderClient, RateLimiterInfo},
    value_objects::AnimeProvider,
};
use crate::shared::{
    errors::{AppError, AppResult},
    utils::RateLimiter,
};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::sync::Arc;

use super::{
    dto::{AniListRequest, AniListResponse, PageResponse},
    graphql::AniListQueries,
    mapper::AniListMapper,
};

pub struct AniListClient {
    client: Client,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
}

impl AniListClient {
    pub fn new() -> AppResult<Self> {
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
