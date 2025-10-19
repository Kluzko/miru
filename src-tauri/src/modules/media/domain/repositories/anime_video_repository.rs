use uuid::Uuid;

use crate::modules::media::domain::entities::{AnimeVideo, NewAnimeVideo};
use crate::modules::media::domain::value_objects::{AnimeProvider, VideoType};

/// Repository trait for anime videos
pub trait AnimeVideoRepository: Send + Sync {
    /// Find video by ID
    fn find_by_id(&self, id: Uuid) -> Result<Option<AnimeVideo>, String>;

    /// Find all videos for an anime
    fn find_by_anime_id(&self, anime_id: Uuid) -> Result<Vec<AnimeVideo>, String>;

    /// Find videos by anime ID and type
    fn find_by_anime_and_type(
        &self,
        anime_id: Uuid,
        video_type: VideoType,
    ) -> Result<Vec<AnimeVideo>, String>;

    /// Find official videos
    fn find_official(&self, anime_id: Uuid) -> Result<Vec<AnimeVideo>, String>;

    /// Find videos by provider
    fn find_by_provider(
        &self,
        anime_id: Uuid,
        provider: AnimeProvider,
    ) -> Result<Vec<AnimeVideo>, String>;

    /// Find promotional videos (trailers, teasers, etc.)
    fn find_promotional(&self, anime_id: Uuid) -> Result<Vec<AnimeVideo>, String>;

    /// Find content videos (openings, endings, clips)
    fn find_content(&self, anime_id: Uuid) -> Result<Vec<AnimeVideo>, String>;

    /// Find videos by site (YouTube, Vimeo, etc.)
    fn find_by_site(&self, anime_id: Uuid, site: &str) -> Result<Vec<AnimeVideo>, String>;

    /// Insert a new video
    fn create(&self, video: NewAnimeVideo) -> Result<AnimeVideo, String>;

    /// Insert multiple videos
    fn create_many(&self, videos: Vec<NewAnimeVideo>) -> Result<Vec<AnimeVideo>, String>;

    /// Update a video
    fn update(&self, id: Uuid, video: NewAnimeVideo) -> Result<AnimeVideo, String>;

    /// Delete a video
    fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Delete all videos for an anime
    fn delete_by_anime_id(&self, anime_id: Uuid) -> Result<usize, String>;

    /// Delete videos by provider
    fn delete_by_provider(&self, anime_id: Uuid, provider: AnimeProvider) -> Result<usize, String>;

    /// Check if video exists by key
    fn exists_by_key(&self, anime_id: Uuid, site: &str, key: &str) -> Result<bool, String>;

    /// Count videos for an anime
    fn count_by_anime_id(&self, anime_id: Uuid) -> Result<i64, String>;
}
