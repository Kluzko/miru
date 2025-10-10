use async_trait::async_trait;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::timeout;

use crate::{
    modules::provider::{
        domain::{entities::AnimeData, repositories::AnimeProviderRepository},
        infrastructure::monitoring::health_monitor::{HealthMonitor, HealthMonitorConfig},
        AnimeProvider,
    },
    shared::errors::{AppError, AppResult},
};

use super::{AniListAdapter, JikanAdapter};

/// Concrete implementation for provider data access
pub struct ProviderRepositoryAdapter {
    anilist_adapter: AniListAdapter,
    jikan_adapter: JikanAdapter,
    health_monitor: Arc<HealthMonitor>,
}

impl ProviderRepositoryAdapter {
    pub fn new() -> Self {
        Self {
            anilist_adapter: AniListAdapter::new(),
            jikan_adapter: JikanAdapter::new(),
            health_monitor: Arc::new(HealthMonitor::new(HealthMonitorConfig::default())),
        }
    }

    pub fn new_with_health_monitor(health_monitor: Arc<HealthMonitor>) -> Self {
        Self {
            anilist_adapter: AniListAdapter::new(),
            jikan_adapter: JikanAdapter::new(),
            health_monitor,
        }
    }

    /// Get the appropriate adapter for a provider
    fn get_adapter(&self, provider: AnimeProvider) -> &dyn ProviderAdapter {
        match provider {
            AnimeProvider::AniList => &self.anilist_adapter,
            AnimeProvider::Jikan => &self.jikan_adapter,
            // For unsupported providers, default to Jikan
            AnimeProvider::Kitsu | AnimeProvider::TMDB | AnimeProvider::AniDB => {
                &self.jikan_adapter
            }
        }
    }
}

#[async_trait]
impl AnimeProviderRepository for ProviderRepositoryAdapter {
    async fn search_anime(
        &self,
        query: &str,
        limit: usize,
        provider: AnimeProvider,
    ) -> AppResult<Vec<AnimeData>> {
        let adapter = self.get_adapter(provider);
        let timeout_duration = Duration::from_secs(10);
        let start_time = Instant::now();

        match timeout(timeout_duration, adapter.search_anime(query, limit)).await {
            Ok(result) => match result {
                Ok(anime_data) => {
                    // Record successful operation
                    let response_time = start_time.elapsed();
                    self.health_monitor
                        .record_success(provider, response_time)
                        .await;
                    Ok(anime_data)
                }
                Err(e) => {
                    // Record failed operation
                    self.health_monitor.record_failure(provider).await;
                    Err(e)
                }
            },
            Err(_) => {
                // Record timeout as failure
                self.health_monitor.record_failure(provider).await;
                Err(AppError::ApiError(format!(
                    "Timeout searching with provider {:?} after {:?}",
                    provider, timeout_duration
                )))
            }
        }
    }

    async fn get_anime_by_id(
        &self,
        id: &str,
        provider: AnimeProvider,
    ) -> AppResult<Option<AnimeData>> {
        let adapter = self.get_adapter(provider);
        let timeout_duration = Duration::from_secs(8);
        let start_time = Instant::now();

        match timeout(timeout_duration, adapter.get_anime_by_id(id)).await {
            Ok(result) => match result {
                Ok(anime_data) => {
                    // Record successful operation
                    let response_time = start_time.elapsed();
                    self.health_monitor
                        .record_success(provider, response_time)
                        .await;
                    Ok(anime_data)
                }
                Err(e) => {
                    // Record failed operation
                    self.health_monitor.record_failure(provider).await;
                    Err(e)
                }
            },
            Err(_) => {
                // Record timeout as failure
                self.health_monitor.record_failure(provider).await;
                Err(AppError::ApiError(format!(
                    "Timeout getting anime by ID with provider {:?} after {:?}",
                    provider, timeout_duration
                )))
            }
        }
    }

    async fn is_provider_available(&self, provider: &AnimeProvider) -> bool {
        // Check if provider is available based on health status
        if let Some(health_metrics) = self.health_monitor.get_provider_health(provider).await {
            // Provider is available if it's not in unhealthy state with too many consecutive failures
            let total_requests = health_metrics.success_count + health_metrics.failure_count;
            if total_requests > 0 {
                let success_rate = health_metrics.success_count as f32 / total_requests as f32;
                success_rate > 0.1 // At least 10% success rate
            } else {
                true // No data yet, assume available
            }
        } else {
            // If no health data available, assume provider is available for first try
            true
        }
    }
}

/// Trait for individual provider adapters
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    // Core anime retrieval functions
    async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeData>>;
    async fn get_anime_by_id(&self, id: &str) -> AppResult<Option<AnimeData>>;
    async fn get_anime(&self, id: u32) -> AppResult<Option<AnimeData>>;
    async fn get_anime_full(&self, id: u32) -> AppResult<Option<AnimeData>>;

    // Search functions
    async fn search_anime_basic(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeData>>;

    // Seasonal content
    async fn get_season_now(&self, limit: usize) -> AppResult<Vec<AnimeData>>;
    async fn get_season_upcoming(&self, limit: usize) -> AppResult<Vec<AnimeData>>;

    // Relations - NOTE: Only AniList provides efficient relationship discovery
    // Other providers should NOT implement this method due to performance limitations
    async fn get_anime_relations(&self, _id: u32) -> AppResult<Vec<(u32, String)>> {
        // Default implementation returns empty for non-AniList providers
        // This is intentional - relationship discovery is AniList-exclusive
        Ok(Vec::new())
    }

    // Provider information
    fn get_provider_type(&self) -> AnimeProvider;
    fn can_make_request_now(&self) -> bool;
}

impl Default for ProviderRepositoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}
