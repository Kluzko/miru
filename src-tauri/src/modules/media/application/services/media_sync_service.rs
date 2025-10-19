use std::sync::Arc;
use uuid::Uuid;

use crate::modules::media::domain::repositories::{AnimeImageRepository, AnimeVideoRepository};
use crate::modules::media::domain::value_objects::AnimeProvider;
use crate::modules::provider::application::service::ProviderService;
use crate::shared::errors::AppResult;
use crate::{log_debug, log_error, log_info};

/// Service for syncing media from providers and storing in database
pub struct MediaSyncService {
    image_repository: Arc<dyn AnimeImageRepository>,
    video_repository: Arc<dyn AnimeVideoRepository>,
    provider_service: Arc<ProviderService>,
}

impl MediaSyncService {
    pub fn new(
        image_repository: Arc<dyn AnimeImageRepository>,
        video_repository: Arc<dyn AnimeVideoRepository>,
        provider_service: Arc<ProviderService>,
    ) -> Self {
        Self {
            image_repository,
            video_repository,
            provider_service,
        }
    }

    /// Sync images and videos from TMDB for an anime
    /// This is the main entry point called when user opens the media tab
    pub async fn sync_media_from_tmdb(
        &self,
        anime_id: Uuid,
        tmdb_id: u32,
        sync_images: bool,
        sync_videos: bool,
    ) -> AppResult<MediaSyncResult> {
        log_info!(
            "Starting TMDB media sync for anime {} (TMDB ID: {})",
            anime_id,
            tmdb_id
        );

        let mut result = MediaSyncResult::default();

        // Sync images if requested
        if sync_images {
            match self.sync_images_from_tmdb(anime_id, tmdb_id).await {
                Ok(count) => {
                    result.images_synced = count;
                    log_info!("Synced {} images from TMDB", count);
                }
                Err(e) => {
                    log_error!("Failed to sync images from TMDB: {}", e);
                    result.errors.push(format!("Images: {}", e));
                }
            }
        }

        // Sync videos if requested
        if sync_videos {
            match self.sync_videos_from_tmdb(anime_id, tmdb_id).await {
                Ok(count) => {
                    result.videos_synced = count;
                    log_info!("Synced {} videos from TMDB", count);
                }
                Err(e) => {
                    log_error!("Failed to sync videos from TMDB: {}", e);
                    result.errors.push(format!("Videos: {}", e));
                }
            }
        }

        log_info!(
            "Completed TMDB media sync: {} images, {} videos",
            result.images_synced,
            result.videos_synced
        );

        Ok(result)
    }

    /// Sync only images from TMDB
    async fn sync_images_from_tmdb(&self, anime_id: Uuid, tmdb_id: u32) -> AppResult<usize> {
        // Check if we already have TMDB images
        let existing_count = self
            .image_repository
            .find_by_provider(anime_id, AnimeProvider::TMDB)
            .map_err(|e| format!("Failed to check existing images: {}", e))?
            .len();

        if existing_count > 0 {
            log_debug!(
                "Anime {} already has {} TMDB images, skipping sync",
                anime_id,
                existing_count
            );
            return Ok(existing_count);
        }

        // Fetch images from provider via ProviderService
        let image_entities = self
            .provider_service
            .fetch_anime_images(tmdb_id, anime_id)
            .await
            .map_err(|e| format!("Provider API error: {}", e))?;

        if image_entities.is_empty() {
            log_debug!("No images found for TMDB ID {}", tmdb_id);
            return Ok(0);
        }

        // Save all images to database
        let saved_images = self
            .image_repository
            .create_many(image_entities)
            .map_err(|e| format!("Failed to save images: {}", e))?;

        Ok(saved_images.len())
    }

    /// Sync only videos from TMDB
    async fn sync_videos_from_tmdb(&self, anime_id: Uuid, tmdb_id: u32) -> AppResult<usize> {
        // Check if we already have TMDB videos
        let existing_count = self
            .video_repository
            .find_by_provider(anime_id, AnimeProvider::TMDB)
            .map_err(|e| format!("Failed to check existing videos: {}", e))?
            .len();

        if existing_count > 0 {
            log_debug!(
                "Anime {} already has {} TMDB videos, skipping sync",
                anime_id,
                existing_count
            );
            return Ok(existing_count);
        }

        // Fetch videos from provider via ProviderService
        let video_entities = self
            .provider_service
            .fetch_anime_videos(tmdb_id, anime_id)
            .await
            .map_err(|e| format!("Provider API error: {}", e))?;

        if video_entities.is_empty() {
            log_debug!("No videos found for TMDB ID {}", tmdb_id);
            return Ok(0);
        }

        // Save videos to database
        let saved_videos = self
            .video_repository
            .create_many(video_entities)
            .map_err(|e| format!("Failed to save videos: {}", e))?;

        Ok(saved_videos.len())
    }

    /// Check if anime already has media from TMDB
    pub fn has_tmdb_media(&self, anime_id: Uuid) -> Result<bool, String> {
        let has_images = !self
            .image_repository
            .find_by_provider(anime_id, AnimeProvider::TMDB)?
            .is_empty();
        let has_videos = !self
            .video_repository
            .find_by_provider(anime_id, AnimeProvider::TMDB)?
            .is_empty();

        Ok(has_images || has_videos)
    }
}

/// Result of media sync operation
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MediaSyncResult {
    pub images_synced: usize,
    pub videos_synced: usize,
    pub errors: Vec<String>,
}

impl MediaSyncResult {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty() && (self.images_synced > 0 || self.videos_synced > 0)
    }
}
