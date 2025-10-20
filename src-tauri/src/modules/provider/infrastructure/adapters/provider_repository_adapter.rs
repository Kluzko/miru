use async_trait::async_trait;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::timeout;
use uuid::Uuid;

use crate::{
    modules::{
        media::domain::entities::{NewAnimeImage, NewAnimeVideo},
        provider::{
            domain::{
                entities::AnimeData,
                repositories::{
                    AnimeProviderRepository, MediaProviderRepository,
                    RelationshipProviderRepository,
                },
            },
            infrastructure::monitoring::health_monitor::{HealthMonitor, HealthMonitorConfig},
            AnimeProvider,
        },
    },
    shared::errors::{AppError, AppResult},
};

use super::{AniListAdapter, JikanAdapter, TmdbAdapter};

/// Concrete implementation for provider data access
pub struct ProviderRepositoryAdapter {
    anilist_adapter: AniListAdapter,
    jikan_adapter: JikanAdapter,
    tmdb_adapter: Option<TmdbAdapter>,
    health_monitor: Arc<HealthMonitor>,
}

impl ProviderRepositoryAdapter {
    pub fn new() -> Self {
        // Load TMDB API key from environment
        let tmdb_adapter = std::env::var("TMBD_API_KEY")
            .ok()
            .map(|api_key| TmdbAdapter::new(api_key));

        if tmdb_adapter.is_none() {
            log::warn!("TMDB adapter not initialized: TMBD_API_KEY not found in environment");
        }

        Self {
            anilist_adapter: AniListAdapter::new(),
            jikan_adapter: JikanAdapter::new(),
            tmdb_adapter,
            health_monitor: Arc::new(HealthMonitor::new(HealthMonitorConfig::default())),
        }
    }

    pub fn new_with_health_monitor(health_monitor: Arc<HealthMonitor>) -> Self {
        // Load TMDB API key from environment
        let tmdb_adapter = std::env::var("TMBD_API_KEY")
            .ok()
            .map(|api_key| TmdbAdapter::new(api_key));

        if tmdb_adapter.is_none() {
            log::warn!("TMDB adapter not initialized: TMBD_API_KEY not found in environment");
        }

        Self {
            anilist_adapter: AniListAdapter::new(),
            jikan_adapter: JikanAdapter::new(),
            tmdb_adapter,
            health_monitor,
        }
    }

    /// Helper to execute search on specific adapter
    async fn search_with_adapter(
        &self,
        query: &str,
        limit: usize,
        provider: AnimeProvider,
    ) -> AppResult<Vec<AnimeData>> {
        match provider {
            AnimeProvider::AniList => self.anilist_adapter.search_anime(query, limit).await,
            AnimeProvider::Jikan => self.jikan_adapter.search_anime(query, limit).await,
            AnimeProvider::TMDB => {
                if let Some(ref tmdb) = self.tmdb_adapter {
                    tmdb.search_anime(query, limit).await
                } else {
                    Err(AppError::ApiError("TMDB adapter not available".to_string()))
                }
            }
            // For unsupported providers, default to Jikan
            AnimeProvider::Kitsu | AnimeProvider::AniDB => {
                self.jikan_adapter.search_anime(query, limit).await
            }
        }
    }

    /// Helper to get anime by ID from specific adapter
    async fn get_by_id_with_adapter(
        &self,
        id: &str,
        provider: AnimeProvider,
    ) -> AppResult<Option<AnimeData>> {
        match provider {
            AnimeProvider::AniList => self.anilist_adapter.get_anime_by_id(id).await,
            AnimeProvider::Jikan => self.jikan_adapter.get_anime_by_id(id).await,
            AnimeProvider::TMDB => {
                if let Some(ref tmdb) = self.tmdb_adapter {
                    tmdb.get_anime_by_id(id).await
                } else {
                    Err(AppError::ApiError("TMDB adapter not available".to_string()))
                }
            }
            // For unsupported providers, default to Jikan
            AnimeProvider::Kitsu | AnimeProvider::AniDB => {
                self.jikan_adapter.get_anime_by_id(id).await
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
        let timeout_duration = Duration::from_secs(10);
        let start_time = Instant::now();

        match timeout(
            timeout_duration,
            self.search_with_adapter(query, limit, provider),
        )
        .await
        {
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
        let timeout_duration = Duration::from_secs(8);
        let start_time = Instant::now();

        match timeout(timeout_duration, self.get_by_id_with_adapter(id, provider)).await {
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

impl Default for ProviderRepositoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// MEDIA PROVIDER REPOSITORY IMPLEMENTATION
// =============================================================================

#[async_trait]
impl MediaProviderRepository for ProviderRepositoryAdapter {
    async fn fetch_images(
        &self,
        provider_anime_id: u32,
        anime_id: Uuid,
    ) -> AppResult<Vec<NewAnimeImage>> {
        // Currently only TMDB supports images
        let tmdb_adapter = self.tmdb_adapter.as_ref().ok_or_else(|| {
            AppError::ApiError("TMDB adapter not available (missing TMBD_API_KEY)".to_string())
        })?;

        let timeout_duration = Duration::from_secs(8);
        let start_time = Instant::now();

        match timeout(timeout_duration, tmdb_adapter.get_images(provider_anime_id)).await {
            Ok(result) => match result {
                Ok(images_response) => {
                    // Record successful operation
                    let response_time = start_time.elapsed();
                    self.health_monitor
                        .record_success(AnimeProvider::TMDB, response_time)
                        .await;

                    // Map images using TmdbMapper
                    use super::tmdb::TmdbMapper;
                    use crate::modules::media::domain::value_objects::ImageType;

                    let mapper = TmdbMapper::new();
                    let mut all_images = Vec::new();

                    // Map posters
                    if let Some(posters) = images_response.posters {
                        all_images.extend(mapper.map_images(posters, anime_id, ImageType::Poster));
                    }

                    // Map backdrops
                    if let Some(backdrops) = images_response.backdrops {
                        all_images.extend(mapper.map_images(
                            backdrops,
                            anime_id,
                            ImageType::Backdrop,
                        ));
                    }

                    // Map logos
                    if let Some(logos) = images_response.logos {
                        all_images.extend(mapper.map_images(logos, anime_id, ImageType::Logo));
                    }

                    Ok(all_images)
                }
                Err(e) => {
                    // Record failed operation
                    self.health_monitor
                        .record_failure(AnimeProvider::TMDB)
                        .await;
                    Err(e)
                }
            },
            Err(_) => {
                // Record timeout as failure
                self.health_monitor
                    .record_failure(AnimeProvider::TMDB)
                    .await;
                Err(AppError::ApiError(format!(
                    "Timeout fetching images from TMDB after {:?}",
                    timeout_duration
                )))
            }
        }
    }

    async fn fetch_videos(
        &self,
        provider_anime_id: u32,
        anime_id: Uuid,
    ) -> AppResult<Vec<NewAnimeVideo>> {
        // Currently only TMDB supports videos
        let tmdb_adapter = self.tmdb_adapter.as_ref().ok_or_else(|| {
            AppError::ApiError("TMDB adapter not available (missing TMBD_API_KEY)".to_string())
        })?;

        let timeout_duration = Duration::from_secs(8);
        let start_time = Instant::now();

        match timeout(timeout_duration, tmdb_adapter.get_videos(provider_anime_id)).await {
            Ok(result) => match result {
                Ok(videos) => {
                    // Record successful operation
                    let response_time = start_time.elapsed();
                    self.health_monitor
                        .record_success(AnimeProvider::TMDB, response_time)
                        .await;

                    // Map videos using TmdbMapper
                    use super::tmdb::TmdbMapper;

                    let mapper = TmdbMapper::new();
                    Ok(mapper.map_videos(videos, anime_id))
                }
                Err(e) => {
                    // Record failed operation
                    self.health_monitor
                        .record_failure(AnimeProvider::TMDB)
                        .await;
                    Err(e)
                }
            },
            Err(_) => {
                // Record timeout as failure
                self.health_monitor
                    .record_failure(AnimeProvider::TMDB)
                    .await;
                Err(AppError::ApiError(format!(
                    "Timeout fetching videos from TMDB after {:?}",
                    timeout_duration
                )))
            }
        }
    }
}

/// Implementation of RelationshipProviderRepository
///
/// Currently delegates all relationship queries to AniList adapter,
/// as AniList provides the most comprehensive relationship data through GraphQL.
#[async_trait]
impl RelationshipProviderRepository for ProviderRepositoryAdapter {
    async fn get_anime_relations(&self, anime_id: u32) -> AppResult<Vec<(u32, String)>> {
        // Delegate to AniList adapter
        self.anilist_adapter
            .get_anime_relations_optimized(anime_id)
            .await
    }

    async fn discover_franchise_details(
        &self,
        anime_id: u32,
    ) -> AppResult<Vec<super::anilist::models::FranchiseRelation>> {
        // Delegate to AniList adapter
        self.anilist_adapter
            .discover_complete_franchise_with_details(anime_id)
            .await
    }

    async fn discover_categorized_franchise(
        &self,
        anime_id: u32,
    ) -> AppResult<super::anilist::models::CategorizedFranchise> {
        // Delegate to AniList adapter
        self.anilist_adapter
            .discover_categorized_franchise(anime_id)
            .await
    }

    fn supports_relationships(&self) -> bool {
        // AniList always supports relationships
        true
    }
}
