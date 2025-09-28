use async_trait::async_trait;

use crate::{
    modules::provider::{domain::entities::AnimeData, AnimeProvider},
    shared::errors::AppResult,
};

/// Repository interface for anime provider data access
/// This defines the contract for fetching anime data from external providers
#[async_trait]
pub trait AnimeProviderRepository: Send + Sync {
    /// Search for anime using a specific provider
    async fn search_anime(
        &self,
        query: &str,
        limit: usize,
        provider: AnimeProvider,
    ) -> AppResult<Vec<AnimeData>>;

    /// Get anime details by ID from a specific provider
    async fn get_anime_by_id(
        &self,
        id: &str,
        provider: AnimeProvider,
    ) -> AppResult<Option<AnimeData>>;

    /// Check if a provider is available/healthy
    async fn is_provider_available(&self, provider: &AnimeProvider) -> bool;
}
