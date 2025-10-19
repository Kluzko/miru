use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::modules::media::domain::entities::{AnimeImage, AnimeVideo};
use crate::modules::media::domain::value_objects::{AnimeProvider, ImageType, VideoType};

/// Image response DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImageResponse {
    pub id: Uuid,
    pub anime_id: Uuid,
    pub provider: AnimeProvider,
    pub provider_image_id: Option<String>,
    pub image_type: ImageType,
    pub is_primary: bool,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub aspect_ratio: Option<f32>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<i32>,
    pub language: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AnimeImage> for ImageResponse {
    fn from(image: AnimeImage) -> Self {
        Self {
            id: image.id,
            anime_id: image.anime_id,
            provider: image.provider,
            provider_image_id: image.provider_image_id,
            image_type: image.image_type,
            is_primary: image.is_primary,
            url: image.url,
            width: image.width,
            height: image.height,
            aspect_ratio: image.aspect_ratio,
            vote_average: image.vote_average,
            vote_count: image.vote_count,
            language: image.language,
            file_size_bytes: image.file_size_bytes,
            created_at: image.created_at,
            updated_at: image.updated_at,
        }
    }
}

/// Video response DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VideoResponse {
    pub id: Uuid,
    pub anime_id: Uuid,
    pub provider: AnimeProvider,
    pub provider_video_id: Option<String>,
    pub video_type: VideoType,
    pub is_official: bool,
    pub name: String,
    pub site: String,
    pub key: String,
    pub url: String,
    pub embed_url: Option<String>,
    pub resolution: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub language: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AnimeVideo> for VideoResponse {
    fn from(video: AnimeVideo) -> Self {
        let embed_url = video.youtube_embed_url();
        Self {
            id: video.id,
            anime_id: video.anime_id,
            provider: video.provider,
            provider_video_id: video.provider_video_id,
            video_type: video.video_type,
            is_official: video.is_official,
            name: video.name,
            site: video.site,
            key: video.key,
            url: video.url,
            embed_url,
            resolution: video.resolution,
            duration_seconds: video.duration_seconds,
            language: video.language,
            published_at: video.published_at,
            created_at: video.created_at,
            updated_at: video.updated_at,
        }
    }
}

/// Grouped media response - all media for an anime
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AnimeMediaResponse {
    pub anime_id: Uuid,
    pub images: MediaImages,
    pub videos: MediaVideos,
}

/// Grouped images by type
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MediaImages {
    pub posters: Vec<ImageResponse>,
    pub backdrops: Vec<ImageResponse>,
    pub logos: Vec<ImageResponse>,
    pub banners: Vec<ImageResponse>,
    pub stills: Vec<ImageResponse>,
    pub covers: Vec<ImageResponse>,
}

/// Grouped videos by type
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MediaVideos {
    pub trailers: Vec<VideoResponse>,
    pub teasers: Vec<VideoResponse>,
    pub clips: Vec<VideoResponse>,
    pub openings: Vec<VideoResponse>,
    pub endings: Vec<VideoResponse>,
    pub pvs: Vec<VideoResponse>,
    pub cms: Vec<VideoResponse>,
    pub behind_the_scenes: Vec<VideoResponse>,
    pub featurettes: Vec<VideoResponse>,
}

impl MediaImages {
    pub fn new() -> Self {
        Self {
            posters: Vec::new(),
            backdrops: Vec::new(),
            logos: Vec::new(),
            banners: Vec::new(),
            stills: Vec::new(),
            covers: Vec::new(),
        }
    }

    pub fn add_image(&mut self, image: ImageResponse) {
        match image.image_type {
            ImageType::Poster => self.posters.push(image),
            ImageType::Backdrop => self.backdrops.push(image),
            ImageType::Logo => self.logos.push(image),
            ImageType::Banner => self.banners.push(image),
            ImageType::Still => self.stills.push(image),
            ImageType::Cover => self.covers.push(image),
        }
    }
}

impl MediaVideos {
    pub fn new() -> Self {
        Self {
            trailers: Vec::new(),
            teasers: Vec::new(),
            clips: Vec::new(),
            openings: Vec::new(),
            endings: Vec::new(),
            pvs: Vec::new(),
            cms: Vec::new(),
            behind_the_scenes: Vec::new(),
            featurettes: Vec::new(),
        }
    }

    pub fn add_video(&mut self, video: VideoResponse) {
        match video.video_type {
            VideoType::Trailer => self.trailers.push(video),
            VideoType::Teaser => self.teasers.push(video),
            VideoType::Clip => self.clips.push(video),
            VideoType::Opening => self.openings.push(video),
            VideoType::Ending => self.endings.push(video),
            VideoType::PV => self.pvs.push(video),
            VideoType::CM => self.cms.push(video),
            VideoType::BehindTheScenes => self.behind_the_scenes.push(video),
            VideoType::Featurette => self.featurettes.push(video),
        }
    }
}
