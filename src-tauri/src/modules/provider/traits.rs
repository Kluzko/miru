use crate::modules::anime::AnimeDetailed;
use crate::modules::provider::AnimeProvider;
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::Duration;

#[async_trait]
pub trait AnimeProviderClient: Send + Sync {
    /// Get the provider type this client handles
    fn provider_type(&self) -> AnimeProvider;

    /// Get rate limiter info from the actual client (single source of truth)
    fn get_rate_limit_info(&self) -> RateLimiterInfo;

    /// Search anime
    async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>>;

    /// Get anime by ID (string format to handle different ID types)
    async fn get_anime_by_id(&self, id: &str) -> AppResult<Option<AnimeDetailed>>;

    /// Get top anime (optional - not all providers may support this)
    async fn get_top_anime(&self, _page: i32, _limit: i32) -> AppResult<Vec<AnimeDetailed>> {
        Err(crate::shared::errors::AppError::NotImplemented(format!(
            "Top anime not supported by {:?}",
            self.provider_type()
        )))
    }

    /// Get seasonal anime (optional - not all providers may support this)
    async fn get_seasonal_anime(
        &self,
        _year: i32,
        _season: &str,
        _page: i32,
    ) -> AppResult<Vec<AnimeDetailed>> {
        Err(crate::shared::errors::AppError::NotImplemented(format!(
            "Seasonal anime not supported by {:?}",
            self.provider_type()
        )))
    }
}

/// Rate limiter information from the actual client implementation
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RateLimiterInfo {
    /// Requests per second
    pub requests_per_second: f64,
    /// Requests per minute (derived)
    pub requests_per_minute: u32,
    /// Minimum delay between requests (in milliseconds)
    pub min_delay_ms: u32,
}

impl RateLimiterInfo {
    pub fn new(requests_per_second: f64) -> Self {
        Self {
            requests_per_second,
            requests_per_minute: (requests_per_second * 60.0) as u32,
            min_delay_ms: ((1.0 / requests_per_second) * 1000.0) as u32,
        }
    }

    pub fn min_delay(&self) -> Duration {
        Duration::from_millis(self.min_delay_ms as u64)
    }
}
