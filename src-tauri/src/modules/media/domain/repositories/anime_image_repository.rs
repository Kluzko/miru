use uuid::Uuid;

use crate::modules::media::domain::entities::{AnimeImage, NewAnimeImage};
use crate::modules::media::domain::value_objects::{AnimeProvider, ImageType};

/// Repository trait for anime images
pub trait AnimeImageRepository: Send + Sync {
    /// Find image by ID
    fn find_by_id(&self, id: Uuid) -> Result<Option<AnimeImage>, String>;

    /// Find all images for an anime
    fn find_by_anime_id(&self, anime_id: Uuid) -> Result<Vec<AnimeImage>, String>;

    /// Find images by anime ID and type
    fn find_by_anime_and_type(
        &self,
        anime_id: Uuid,
        image_type: ImageType,
    ) -> Result<Vec<AnimeImage>, String>;

    /// Find primary image for an anime by type
    fn find_primary_by_type(
        &self,
        anime_id: Uuid,
        image_type: ImageType,
    ) -> Result<Option<AnimeImage>, String>;

    /// Find all primary images for an anime
    fn find_all_primary(&self, anime_id: Uuid) -> Result<Vec<AnimeImage>, String>;

    /// Find images by provider
    fn find_by_provider(
        &self,
        anime_id: Uuid,
        provider: AnimeProvider,
    ) -> Result<Vec<AnimeImage>, String>;

    /// Find best quality images by type
    fn find_best_quality(
        &self,
        anime_id: Uuid,
        image_type: ImageType,
        limit: i64,
    ) -> Result<Vec<AnimeImage>, String>;

    /// Insert a new image
    fn create(&self, image: NewAnimeImage) -> Result<AnimeImage, String>;

    /// Insert multiple images
    fn create_many(&self, images: Vec<NewAnimeImage>) -> Result<Vec<AnimeImage>, String>;

    /// Update an image
    fn update(&self, id: Uuid, image: NewAnimeImage) -> Result<AnimeImage, String>;

    /// Delete an image
    fn delete(&self, id: Uuid) -> Result<bool, String>;

    /// Delete all images for an anime
    fn delete_by_anime_id(&self, anime_id: Uuid) -> Result<usize, String>;

    /// Delete images by provider
    fn delete_by_provider(&self, anime_id: Uuid, provider: AnimeProvider) -> Result<usize, String>;

    /// Set primary image (unsets other primary images of same type)
    fn set_primary(&self, id: Uuid) -> Result<AnimeImage, String>;

    /// Check if image exists by URL
    fn exists_by_url(&self, anime_id: Uuid, url: &str) -> Result<bool, String>;

    /// Count images for an anime
    fn count_by_anime_id(&self, anime_id: Uuid) -> Result<i64, String>;
}
