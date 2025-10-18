use async_trait::async_trait;

use crate::modules::anime::domain::{AnimeDetailed, AnimeRelation};
use crate::shared::{domain::value_objects::AnimeProvider, errors::AppResult};

/// Port (interface) for external provider clients
/// Infrastructure layer implements this for each provider (Jikan, AniList, etc.)
#[async_trait]
pub trait ProviderClient: Send + Sync {
    /// Get the provider type
    fn provider(&self) -> AnimeProvider;

    /// Fetch anime details from external provider
    async fn fetch_anime(&self, external_id: &str) -> AppResult<AnimeDetailed>;

    /// Fetch relations for an anime from external provider
    async fn fetch_relations(&self, external_id: &str) -> AppResult<Vec<AnimeRelation>>;

    /// Search anime on external provider
    async fn search(&self, query: &str, limit: u32) -> AppResult<Vec<AnimeDetailed>>;
}
