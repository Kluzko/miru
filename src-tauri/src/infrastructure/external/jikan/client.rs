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
use std::sync::Arc;

use super::{
    dto::{JikanAnimeListResponse, JikanAnimeResponse, JikanSearchParams},
    mapper::JikanMapper,
};

pub struct JikanClient {
    client: Client,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
}

impl JikanClient {
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
            base_url: "https://api.jikan.moe/v4".to_string(),
            rate_limiter: Arc::new(RateLimiter::new(3.0)), // 3 requests per second for Jikan (official limit)
        })
    }

    pub async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        if query.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Search query cannot be empty".to_string(),
            ));
        }

        self.rate_limiter.wait().await?;

        let params = JikanSearchParams {
            q: Some(query.trim().to_string()),
            limit: Some((limit.min(25)) as i32), // Jikan max limit is 25
            sfw: Some(true),
            ..Default::default()
        };

        let url = format!("{}/anime", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| AppError::ApiError(format!("Jikan search failed: {}", e)))?;

        self.handle_response_status(response.status())?;

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
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ApiError(format!("Jikan get anime failed: {}", e)))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        self.handle_response_status(response.status())?;

        let jikan_response = response
            .json::<JikanAnimeResponse>()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to parse Jikan response: {}", e)))?;

        Ok(Some(JikanMapper::to_domain(jikan_response.data)))
    }

    pub async fn get_top_anime(&self, page: i32, limit: i32) -> AppResult<Vec<AnimeDetailed>> {
        self.rate_limiter.wait().await?;

        let url = format!("{}/top/anime", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[("page", page.to_string()), ("limit", limit.to_string())])
            .send()
            .await
            .map_err(|e| AppError::ApiError(format!("Jikan get top anime failed: {}", e)))?;

        self.handle_response_status(response.status())?;

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
        let response = self
            .client
            .get(&url)
            .query(&[("page", page.to_string())])
            .send()
            .await
            .map_err(|e| AppError::ApiError(format!("Jikan get seasonal anime failed: {}", e)))?;

        self.handle_response_status(response.status())?;

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

    fn handle_response_status(&self, status: StatusCode) -> AppResult<()> {
        match status {
            StatusCode::OK => Ok(()),
            StatusCode::TOO_MANY_REQUESTS => Err(AppError::RateLimitError(
                "Jikan rate limit exceeded".to_string(),
            )),
            StatusCode::NOT_FOUND => Err(AppError::NotFound("Resource not found".to_string())),
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::SERVICE_UNAVAILABLE => Err(
                AppError::ExternalServiceError("Jikan service unavailable".to_string()),
            ),
            _ => Err(AppError::ApiError(format!(
                "Unexpected status code: {}",
                status
            ))),
        }
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
