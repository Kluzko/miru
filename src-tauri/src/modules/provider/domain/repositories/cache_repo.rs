use async_trait::async_trait;

use crate::modules::provider::{domain::entities::AnimeData, AnimeProvider};

/// Repository interface for caching anime data
/// This abstracts the caching mechanism from business logic
#[async_trait]
pub trait CacheRepository: Send + Sync {
    /// Get cached search results
    async fn get_search_results(
        &self,
        query: &str,
        provider: AnimeProvider,
    ) -> Option<Vec<AnimeData>>;

    /// Cache search results
    async fn cache_search_results(
        &self,
        query: &str,
        provider: AnimeProvider,
        results: Vec<AnimeData>,
    );

    /// Get cached anime details
    async fn get_anime_details(&self, id: &str, provider: AnimeProvider) -> Option<AnimeData>;

    /// Cache anime details
    async fn cache_anime_details(&self, id: &str, provider: AnimeProvider, anime: AnimeData);

    /// Clear all cached data
    async fn clear_cache(&self);

    /// Get cache statistics
    async fn get_cache_stats(&self) -> CacheStats;
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct CacheStats {
    pub search_entries: usize,
    pub details_entries: usize,
    pub total_entries: usize,
    pub hit_rate: f32,
    pub miss_rate: f32,
    pub search_ttl_seconds: u32,
    pub details_ttl_seconds: u32,
}
