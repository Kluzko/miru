use crate::modules::media::domain::entities::{NewAnimeImage, NewAnimeVideo};
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository interface for fetching media data from external providers
///
/// This defines the contract for fetching anime images and videos from
/// external providers like TMDB. Implementations hide provider-specific
/// details and provide a clean abstraction for the application layer.
#[async_trait]
pub trait MediaProviderRepository: Send + Sync {
    /// Fetch images for an anime from a provider
    ///
    /// # Arguments
    /// * `provider_anime_id` - The anime ID in the provider's system (e.g., TMDB ID)
    /// * `anime_id` - The UUID of the anime in our database
    ///
    /// # Returns
    /// Vector of NewAnimeImage entities ready for database insertion
    async fn fetch_images(
        &self,
        provider_anime_id: u32,
        anime_id: Uuid,
    ) -> AppResult<Vec<NewAnimeImage>>;

    /// Fetch videos for an anime from a provider
    ///
    /// # Arguments
    /// * `provider_anime_id` - The anime ID in the provider's system (e.g., TMDB ID)
    /// * `anime_id` - The UUID of the anime in our database
    ///
    /// # Returns
    /// Vector of NewAnimeVideo entities ready for database insertion
    async fn fetch_videos(
        &self,
        provider_anime_id: u32,
        anime_id: Uuid,
    ) -> AppResult<Vec<NewAnimeVideo>>;
}
