use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

use crate::modules::media::application::dto::*;
use crate::modules::media::application::services::{
    MediaService, MediaStats, MediaSyncResult, MediaSyncService,
};

/// Get all media for an anime (images and videos grouped by type)
#[tauri::command]
#[specta::specta]
pub async fn get_anime_media(
    request: GetAnimeMediaRequest,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<AnimeMediaResponse, String> {
    media_service.get_anime_media(request.anime_id)
}

/// Get images for an anime with optional filters
#[tauri::command]
#[specta::specta]
pub async fn get_anime_images(
    request: GetAnimeImagesRequest,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<Vec<ImageResponse>, String> {
    media_service.get_anime_images(request.anime_id, request.image_type, request.provider)
}

/// Get videos for an anime with optional filters
#[tauri::command]
#[specta::specta]
pub async fn get_anime_videos(
    request: GetAnimeVideosRequest,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<Vec<VideoResponse>, String> {
    media_service.get_anime_videos(request.anime_id, request.video_type, request.provider)
}

/// Get primary images for an anime (one per type)
#[tauri::command]
#[specta::specta]
pub async fn get_primary_images(
    request: GetPrimaryImagesRequest,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<Vec<ImageResponse>, String> {
    media_service.get_primary_images(request.anime_id)
}

/// Get best quality images by type
#[tauri::command]
#[specta::specta]
pub async fn get_best_quality_images(
    request: GetBestQualityImagesRequest,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<Vec<ImageResponse>, String> {
    media_service.get_best_quality_images(request.anime_id, request.image_type, request.limit)
}

/// Get official videos for an anime
#[tauri::command]
#[specta::specta]
pub async fn get_official_videos(
    anime_id: Uuid,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<Vec<VideoResponse>, String> {
    media_service.get_official_videos(anime_id)
}

/// Get promotional videos (trailers, teasers, PVs, etc.)
#[tauri::command]
#[specta::specta]
pub async fn get_promotional_videos(
    anime_id: Uuid,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<Vec<VideoResponse>, String> {
    media_service.get_promotional_videos(anime_id)
}

/// Get content videos (openings, endings, clips)
#[tauri::command]
#[specta::specta]
pub async fn get_content_videos(
    anime_id: Uuid,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<Vec<VideoResponse>, String> {
    media_service.get_content_videos(anime_id)
}

/// Set an image as primary for its type
#[tauri::command]
#[specta::specta]
pub async fn set_primary_image(
    request: SetPrimaryImageRequest,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<ImageResponse, String> {
    media_service.set_primary_image(request.image_id)
}

/// Delete all media from a specific provider for an anime
#[tauri::command]
#[specta::specta]
pub async fn delete_media_by_provider(
    request: DeleteMediaByProviderRequest,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<DeleteMediaResponse, String> {
    let (images_deleted, videos_deleted) =
        media_service.delete_media_by_provider(request.anime_id, request.provider)?;

    Ok(DeleteMediaResponse {
        images_deleted,
        videos_deleted,
    })
}

/// Get media statistics for an anime
#[tauri::command]
#[specta::specta]
pub async fn get_media_stats(
    anime_id: Uuid,
    media_service: State<'_, Arc<MediaService>>,
) -> Result<MediaStats, String> {
    media_service.get_media_stats(anime_id)
}

/// Sync media from provider (fetch and save to database)
/// This is called when user opens the media tab for the first time
#[tauri::command]
#[specta::specta]
pub async fn sync_media_from_provider(
    request: SyncMediaRequest,
    sync_service: State<'_, Arc<MediaSyncService>>,
) -> Result<MediaSyncResult, String> {
    sync_service
        .sync_media_from_tmdb(
            request.anime_id,
            request.tmdb_id,
            request.sync_images,
            request.sync_videos,
        )
        .await
        .map_err(|e| e.to_string())
}

/// Check if anime already has media from a provider
#[tauri::command]
#[specta::specta]
pub async fn has_provider_media(
    anime_id: Uuid,
    sync_service: State<'_, Arc<MediaSyncService>>,
) -> Result<bool, String> {
    sync_service.has_tmdb_media(anime_id)
}

/// Delete media response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMediaResponse {
    pub images_deleted: usize,
    pub videos_deleted: usize,
}
