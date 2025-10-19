use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::modules::media::domain::value_objects::{AnimeProvider, ImageType, VideoType};

/// Get media for an anime
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GetAnimeMediaRequest {
    pub anime_id: Uuid,
}

/// Get images for an anime
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GetAnimeImagesRequest {
    pub anime_id: Uuid,
    pub image_type: Option<ImageType>,
    pub provider: Option<AnimeProvider>,
}

/// Get videos for an anime
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GetAnimeVideosRequest {
    pub anime_id: Uuid,
    pub video_type: Option<VideoType>,
    pub provider: Option<AnimeProvider>,
}

/// Get primary images for an anime
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GetPrimaryImagesRequest {
    pub anime_id: Uuid,
}

/// Get best quality images by type
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GetBestQualityImagesRequest {
    pub anime_id: Uuid,
    pub image_type: ImageType,
    pub limit: Option<i32>,
}

/// Sync media from provider
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SyncMediaRequest {
    pub anime_id: Uuid,
    pub tmdb_id: u32, // Provider-specific ID (TMDB ID in this case)
    pub sync_images: bool,
    pub sync_videos: bool,
}

/// Set primary image
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetPrimaryImageRequest {
    pub image_id: Uuid,
}

/// Delete media by provider
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMediaByProviderRequest {
    pub anime_id: Uuid,
    pub provider: AnimeProvider,
}
