use std::sync::Arc;
use uuid::Uuid;

use crate::modules::media::application::dto::{
    AnimeMediaResponse, ImageResponse, MediaImages, MediaVideos, VideoResponse,
};
use crate::modules::media::domain::repositories::{AnimeImageRepository, AnimeVideoRepository};
use crate::modules::media::domain::value_objects::{AnimeProvider, ImageType, VideoType};

/// Media application service
pub struct MediaService {
    image_repository: Arc<dyn AnimeImageRepository>,
    video_repository: Arc<dyn AnimeVideoRepository>,
}

impl MediaService {
    pub fn new(
        image_repository: Arc<dyn AnimeImageRepository>,
        video_repository: Arc<dyn AnimeVideoRepository>,
    ) -> Self {
        Self {
            image_repository,
            video_repository,
        }
    }

    /// Get all media for an anime (grouped by type)
    pub fn get_anime_media(&self, anime_id: Uuid) -> Result<AnimeMediaResponse, String> {
        let images = self.image_repository.find_by_anime_id(anime_id)?;
        let videos = self.video_repository.find_by_anime_id(anime_id)?;

        let mut media_images = MediaImages::new();
        for image in images {
            media_images.add_image(image.into());
        }

        let mut media_videos = MediaVideos::new();
        for video in videos {
            media_videos.add_video(video.into());
        }

        Ok(AnimeMediaResponse {
            anime_id,
            images: media_images,
            videos: media_videos,
        })
    }

    /// Get images for an anime (with optional filters)
    pub fn get_anime_images(
        &self,
        anime_id: Uuid,
        image_type: Option<ImageType>,
        provider: Option<AnimeProvider>,
    ) -> Result<Vec<ImageResponse>, String> {
        let images = match (image_type, provider) {
            (Some(img_type), None) => self
                .image_repository
                .find_by_anime_and_type(anime_id, img_type)?,
            (None, Some(prov)) => self.image_repository.find_by_provider(anime_id, prov)?,
            (Some(_), Some(_)) => {
                // If both filters, get all and filter in memory
                let all_images = self.image_repository.find_by_anime_id(anime_id)?;
                all_images
                    .into_iter()
                    .filter(|img| {
                        image_type.map_or(true, |t| img.image_type == t)
                            && provider.map_or(true, |p| img.provider == p)
                    })
                    .collect()
            }
            (None, None) => self.image_repository.find_by_anime_id(anime_id)?,
        };

        Ok(images.into_iter().map(Into::into).collect())
    }

    /// Get videos for an anime (with optional filters)
    pub fn get_anime_videos(
        &self,
        anime_id: Uuid,
        video_type: Option<VideoType>,
        provider: Option<AnimeProvider>,
    ) -> Result<Vec<VideoResponse>, String> {
        let videos = match (video_type, provider) {
            (Some(vid_type), None) => self
                .video_repository
                .find_by_anime_and_type(anime_id, vid_type)?,
            (None, Some(prov)) => self.video_repository.find_by_provider(anime_id, prov)?,
            (Some(_), Some(_)) => {
                // If both filters, get all and filter in memory
                let all_videos = self.video_repository.find_by_anime_id(anime_id)?;
                all_videos
                    .into_iter()
                    .filter(|vid| {
                        video_type.map_or(true, |t| vid.video_type == t)
                            && provider.map_or(true, |p| vid.provider == p)
                    })
                    .collect()
            }
            (None, None) => self.video_repository.find_by_anime_id(anime_id)?,
        };

        Ok(videos.into_iter().map(Into::into).collect())
    }

    /// Get primary images for an anime (one per type)
    pub fn get_primary_images(&self, anime_id: Uuid) -> Result<Vec<ImageResponse>, String> {
        let images = self.image_repository.find_all_primary(anime_id)?;
        Ok(images.into_iter().map(Into::into).collect())
    }

    /// Get best quality images by type
    pub fn get_best_quality_images(
        &self,
        anime_id: Uuid,
        image_type: ImageType,
        limit: Option<i32>,
    ) -> Result<Vec<ImageResponse>, String> {
        let limit = limit.unwrap_or(10).max(1).min(50) as i64; // Clamp between 1-50
        let images = self
            .image_repository
            .find_best_quality(anime_id, image_type, limit)?;
        Ok(images.into_iter().map(Into::into).collect())
    }

    /// Get official videos for an anime
    pub fn get_official_videos(&self, anime_id: Uuid) -> Result<Vec<VideoResponse>, String> {
        let videos = self.video_repository.find_official(anime_id)?;
        Ok(videos.into_iter().map(Into::into).collect())
    }

    /// Get promotional videos (trailers, teasers, etc.)
    pub fn get_promotional_videos(&self, anime_id: Uuid) -> Result<Vec<VideoResponse>, String> {
        let videos = self.video_repository.find_promotional(anime_id)?;
        Ok(videos.into_iter().map(Into::into).collect())
    }

    /// Get content videos (openings, endings, clips)
    pub fn get_content_videos(&self, anime_id: Uuid) -> Result<Vec<VideoResponse>, String> {
        let videos = self.video_repository.find_content(anime_id)?;
        Ok(videos.into_iter().map(Into::into).collect())
    }

    /// Set an image as primary for its type
    pub fn set_primary_image(&self, image_id: Uuid) -> Result<ImageResponse, String> {
        let image = self.image_repository.set_primary(image_id)?;
        Ok(image.into())
    }

    /// Delete all media for an anime from a specific provider
    pub fn delete_media_by_provider(
        &self,
        anime_id: Uuid,
        provider: AnimeProvider,
    ) -> Result<(usize, usize), String> {
        let images_deleted = self
            .image_repository
            .delete_by_provider(anime_id, provider)?;
        let videos_deleted = self
            .video_repository
            .delete_by_provider(anime_id, provider)?;
        Ok((images_deleted, videos_deleted))
    }

    /// Get media statistics for an anime
    pub fn get_media_stats(&self, anime_id: Uuid) -> Result<MediaStats, String> {
        let image_count = self.image_repository.count_by_anime_id(anime_id)?;
        let video_count = self.video_repository.count_by_anime_id(anime_id)?;

        Ok(MediaStats {
            anime_id,
            total_images: image_count,
            total_videos: video_count,
        })
    }
}

/// Media statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MediaStats {
    pub anime_id: Uuid,
    pub total_images: i64,
    pub total_videos: i64,
}
