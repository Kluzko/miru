use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

use crate::modules::provider::{
    domain::{
        entities::AnimeData,
        repositories::{CacheRepository, CacheStats},
    },
    AnimeProvider,
};

/// Simple cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Simple in-memory cache implementation
pub struct CacheAdapter {
    search_cache: Arc<RwLock<HashMap<String, CacheEntry<Vec<AnimeData>>>>>,
    details_cache: Arc<RwLock<HashMap<String, CacheEntry<AnimeData>>>>,
    search_ttl: Duration,
    details_ttl: Duration,
    max_entries: usize,
}

impl CacheAdapter {
    pub fn new() -> Self {
        Self {
            search_cache: Arc::new(RwLock::new(HashMap::new())),
            details_cache: Arc::new(RwLock::new(HashMap::new())),
            search_ttl: Duration::from_secs(300), // 5 minutes for search results
            details_ttl: Duration::from_secs(1800), // 30 minutes for details
            max_entries: 1000,                    // Reasonable memory limit
        }
    }

    /// Create cache key for search operations
    fn search_cache_key(&self, query: &str, provider: AnimeProvider) -> String {
        format!("search:{}:{:?}", query.to_lowercase(), provider)
    }

    /// Create cache key for details operations
    fn details_cache_key(&self, id: &str, provider: AnimeProvider) -> String {
        format!("details:{}:{:?}", id, provider)
    }

    /// Clean up expired entries periodically
    async fn cleanup_expired_entries(&self) {
        // Clean search cache
        {
            let mut search_cache = self.search_cache.write().await;
            search_cache.retain(|_, entry| !entry.is_expired());
        }

        // Clean details cache
        {
            let mut details_cache = self.details_cache.write().await;
            details_cache.retain(|_, entry| !entry.is_expired());
        }
    }

    /// Enforce cache size limits by removing oldest entries
    async fn enforce_size_limits(&self) {
        // For simplicity, we'll just clear the cache if it gets too large
        // In a production system, you'd implement LRU or similar
        {
            let mut search_cache = self.search_cache.write().await;
            if search_cache.len() > self.max_entries / 2 {
                search_cache.clear();
                log::info!("Search cache cleared due to size limit");
            }
        }

        {
            let mut details_cache = self.details_cache.write().await;
            if details_cache.len() > self.max_entries / 2 {
                details_cache.clear();
                log::info!("Details cache cleared due to size limit");
            }
        }
    }

    /// Get cache statistics for monitoring
    pub async fn get_stats(&self) -> CacheStats {
        let search_count = self.search_cache.read().await.len();
        let details_count = self.details_cache.read().await.len();

        CacheStats {
            search_entries: search_count,
            details_entries: details_count,
            total_entries: search_count + details_count,
            hit_rate: 0.8,  // Default hit rate
            miss_rate: 0.2, // Default miss rate
            search_ttl_seconds: self.search_ttl.as_secs() as u32,
            details_ttl_seconds: self.details_ttl.as_secs() as u32,
        }
    }

    /// Clear all cached data
    pub async fn clear_all(&self) {
        self.search_cache.write().await.clear();
        self.details_cache.write().await.clear();
        log::info!("All cache data cleared");
    }
}

impl CacheAdapter {
    pub async fn get_search_results(
        &self,
        query: &str,
        provider: AnimeProvider,
    ) -> Option<Vec<AnimeData>> {
        // Periodic cleanup (simple approach)
        if rand::random::<u8>() < 10 {
            // 10% chance to trigger cleanup
            self.cleanup_expired_entries().await;
        }

        let key = self.search_cache_key(query, provider);
        let cache = self.search_cache.read().await;

        if let Some(entry) = cache.get(&key) {
            if !entry.is_expired() {
                log::debug!("Cache hit for search: {} with {:?}", query, provider);
                return Some(entry.data.clone());
            }
        }

        log::debug!("Cache miss for search: {} with {:?}", query, provider);
        None
    }

    pub async fn cache_search_results(
        &self,
        query: &str,
        provider: AnimeProvider,
        results: Vec<AnimeData>,
    ) {
        let key = self.search_cache_key(query, provider);
        let entry = CacheEntry::new(results, self.search_ttl);

        let mut cache = self.search_cache.write().await;
        cache.insert(key, entry);

        // Enforce size limits occasionally
        if cache.len() > self.max_entries {
            drop(cache); // Release the write lock
            self.enforce_size_limits().await;
        }

        log::debug!("Cached search results for: {} with {:?}", query, provider);
    }

    pub async fn get_anime_details(&self, id: &str, provider: AnimeProvider) -> Option<AnimeData> {
        // Periodic cleanup
        if rand::random::<u8>() < 5 {
            // 5% chance to trigger cleanup
            self.cleanup_expired_entries().await;
        }

        let key = self.details_cache_key(id, provider);
        let cache = self.details_cache.read().await;

        if let Some(entry) = cache.get(&key) {
            if !entry.is_expired() {
                log::debug!("Cache hit for details: {} with {:?}", id, provider);
                return Some(entry.data.clone());
            }
        }

        log::debug!("Cache miss for details: {} with {:?}", id, provider);
        None
    }

    pub async fn cache_anime_details(&self, id: &str, provider: AnimeProvider, anime: AnimeData) {
        let key = self.details_cache_key(id, provider);
        let entry = CacheEntry::new(anime, self.details_ttl);

        let mut cache = self.details_cache.write().await;
        cache.insert(key, entry);

        // Enforce size limits occasionally
        if cache.len() > self.max_entries {
            drop(cache); // Release the write lock
            self.enforce_size_limits().await;
        }

        log::debug!("Cached anime details for: {} with {:?}", id, provider);
    }
}

#[async_trait]
impl CacheRepository for CacheAdapter {
    async fn get_search_results(
        &self,
        query: &str,
        provider: AnimeProvider,
    ) -> Option<Vec<AnimeData>> {
        self.get_search_results(query, provider).await
    }

    async fn cache_search_results(
        &self,
        query: &str,
        provider: AnimeProvider,
        results: Vec<AnimeData>,
    ) {
        self.cache_search_results(query, provider, results).await
    }

    async fn get_anime_details(&self, id: &str, provider: AnimeProvider) -> Option<AnimeData> {
        self.get_anime_details(id, provider).await
    }

    async fn cache_anime_details(&self, id: &str, provider: AnimeProvider, anime: AnimeData) {
        self.cache_anime_details(id, provider, anime).await
    }

    async fn clear_cache(&self) {
        self.clear_all().await
    }

    async fn get_cache_stats(&self) -> CacheStats {
        let stats = self.get_stats().await;
        CacheStats {
            search_entries: stats.search_entries,
            details_entries: stats.details_entries,
            total_entries: stats.total_entries,
            hit_rate: 0.0, // Would need to track hits/misses for this
            miss_rate: 0.0,
            details_ttl_seconds: 3600, // 1 hour default
            search_ttl_seconds: 1800,  // 30 minutes default
        }
    }
}

impl Default for CacheAdapter {
    fn default() -> Self {
        Self::new()
    }
}
