use crate::domain::entities::Anime;
use crate::shared::{
    errors::{AppError, AppResult},
    utils::RateLimiter,
};
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
            rate_limiter: Arc::new(RateLimiter::new(1.0)), // 1 request per second for Jikan
        })
    }

    pub async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<Anime>> {
        self.rate_limiter.wait().await?;

        let params = JikanSearchParams {
            q: Some(query.to_string()),
            limit: Some(limit as i32),
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

    pub async fn get_anime_by_id(&self, mal_id: i32) -> AppResult<Option<Anime>> {
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

    pub async fn get_top_anime(&self, page: i32, limit: i32) -> AppResult<Vec<Anime>> {
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
    ) -> AppResult<Vec<Anime>> {
        self.rate_limiter.wait().await?;

        let url = format!(
            "{}/seasons/{}/{}",
            self.base_url,
            year,
            season.to_lowercase()
        );
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

    pub async fn get_anime_recommendations(&self, mal_id: i32) -> AppResult<Vec<Anime>> {
        self.rate_limiter.wait().await?;

        let url = format!("{}/anime/{}/recommendations", self.base_url, mal_id);
        let response =
            self.client.get(&url).send().await.map_err(|e| {
                AppError::ApiError(format!("Jikan get recommendations failed: {}", e))
            })?;

        self.handle_response_status(response.status())?;

        // Note: The recommendations endpoint returns a different structure
        // You might need to create a separate DTO for this
        // For now, returning empty vec as placeholder
        Ok(Vec::new())
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
