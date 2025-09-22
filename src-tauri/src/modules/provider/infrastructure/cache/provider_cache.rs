use crate::modules::anime::AnimeDetailed;
use crate::modules::provider::AnimeProvider;
use crate::shared::errors::AppResult;
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Cached entry with TTL support
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<AnimeDetailed>,
    created_at: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn new(data: Vec<AnimeDetailed>, ttl: Duration) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries_count: usize,
    pub expired_cleanups: u64,
    pub cache_size_bytes: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
}

/// Provider response cache with TTL support and background cleanup
#[derive(Debug)]
pub struct ProviderCache {
    cache: Arc<DashMap<String, CacheEntry>>,
    stats: Arc<RwLock<CacheStats>>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    cleanups: Arc<AtomicU64>,
    cleanup_task_started: Arc<AtomicBool>,
    default_ttl: Duration,
    not_found_ttl: Duration,
    max_entries: usize,
}

impl ProviderCache {
    /// Create a new provider cache with specified configurations
    pub fn new(default_ttl_minutes: u64, not_found_ttl_minutes: u64, max_entries: usize) -> Self {
        let cache = Arc::new(DashMap::new());

        let provider_cache = Self {
            cache: cache.clone(),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            cleanups: Arc::new(AtomicU64::new(0)),
            cleanup_task_started: Arc::new(AtomicBool::new(false)),
            default_ttl: Duration::from_secs(default_ttl_minutes * 60),
            not_found_ttl: Duration::from_secs(not_found_ttl_minutes * 60),
            max_entries,
        };

        // Try to start cleanup task immediately if tokio runtime is available
        // If not available, it will be started on first cache operation
        if tokio::runtime::Handle::try_current().is_ok() {
            provider_cache.ensure_cleanup_task_started();
        }

        provider_cache
    }

    /// Create cache with default settings (5 minutes default, 2 minutes for not found, 1000 max entries)
    pub fn default() -> Self {
        Self::new(5, 2, 1000)
    }

    /// Generate cache key with provider prefix and normalized query (optimized for single allocation)
    fn generate_cache_key(&self, provider: &AnimeProvider, query: &str) -> String {
        let provider_str = match provider {
            AnimeProvider::Jikan => "jikan",
            AnimeProvider::AniList => "anilist",
            AnimeProvider::Kitsu => "kitsu",
            AnimeProvider::TMDB => "tmdb",
            AnimeProvider::AniDB => "anidb",
        };

        // Single allocation: directly format normalized query with provider prefix
        let trimmed = query.trim();
        let mut result = String::with_capacity(provider_str.len() + 1 + trimmed.len());
        result.push_str(provider_str);
        result.push(':');

        // Append lowercase characters directly to avoid intermediate string
        for ch in trimmed.chars() {
            result.extend(ch.to_lowercase());
        }

        result
    }

    /// Get cached search results if available and not expired
    pub async fn get_search_results(
        &self,
        provider: &AnimeProvider,
        query: &str,
    ) -> Option<Vec<AnimeDetailed>> {
        // Ensure cleanup task is started when runtime is available
        self.ensure_cleanup_task_started();

        let key = self.generate_cache_key(provider, query);

        if let Some(entry) = self.cache.get(&key) {
            if !entry.is_expired() {
                self.hits.fetch_add(1, Ordering::Relaxed);
                debug!("Cache hit for key: {}", key);
                return Some(entry.data.clone());
            } else {
                // Remove expired entry
                self.cache.remove(&key);
                debug!("Removed expired cache entry for key: {}", key);
            }
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        debug!("Cache miss for key: {}", key);
        None
    }

    /// Cache search results with appropriate TTL
    pub async fn cache_search_results(
        &self,
        provider: &AnimeProvider,
        query: &str,
        results: Vec<AnimeDetailed>,
    ) -> AppResult<()> {
        // Ensure cleanup task is started when runtime is available
        self.ensure_cleanup_task_started();

        // Check if we need to evict entries to stay under max_entries limit
        if self.cache.len() >= self.max_entries {
            self.evict_oldest_entries().await;
        }

        let key = self.generate_cache_key(provider, query);
        let ttl = if results.is_empty() {
            self.not_found_ttl
        } else {
            self.default_ttl
        };

        let entry = CacheEntry::new(results, ttl);
        self.cache.insert(key.clone(), entry);

        debug!("Cached results for key: {} with TTL: {:?}", key, ttl);
        Ok(())
    }

    /// Check if a request is already being processed to prevent duplicate concurrent requests
    pub async fn is_request_in_progress(&self, provider: &AnimeProvider, query: &str) -> bool {
        let key = format!("{}_in_progress", self.generate_cache_key(provider, query));
        self.cache.contains_key(&key)
    }

    /// Mark a request as in progress
    pub async fn mark_request_in_progress(&self, provider: &AnimeProvider, query: &str) {
        let key = format!("{}_in_progress", self.generate_cache_key(provider, query));
        let entry = CacheEntry::new(vec![], Duration::from_secs(30)); // Short TTL for in-progress markers
        self.cache.insert(key, entry);
    }

    /// Remove in-progress marker
    pub async fn remove_request_in_progress(&self, provider: &AnimeProvider, query: &str) {
        let key = format!("{}_in_progress", self.generate_cache_key(provider, query));
        self.cache.remove(&key);
    }

    /// Get current cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let cleanups = self.cleanups.load(Ordering::Relaxed);
        let entries_count = self.cache.len();

        // Estimate cache size (rough approximation)
        let cache_size_bytes = entries_count * 1024; // Rough estimate: 1KB per entry

        CacheStats {
            hits,
            misses,
            entries_count,
            expired_cleanups: cleanups,
            cache_size_bytes,
        }
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        self.cache.clear();
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.cleanups.store(0, Ordering::Relaxed);
        info!("Cache cleared");
    }

    /// Ensure cleanup task is started (idempotent)
    fn ensure_cleanup_task_started(&self) {
        // Check if task is already started using atomic compare-and-swap
        if !self
            .cleanup_task_started
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return; // Task already started
        }

        self.start_cleanup_task();
        debug!("Background cleanup task started");
    }

    /// Start background task for cleaning up expired entries
    fn start_cleanup_task(&self) {
        let cache = self.cache.clone();
        let cleanups = self.cleanups.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes

            loop {
                interval.tick().await;

                let mut expired_keys = Vec::new();
                for entry in cache.iter() {
                    if entry.value().is_expired() {
                        expired_keys.push(entry.key().clone());
                    }
                }

                let expired_count = expired_keys.len();
                for key in expired_keys {
                    cache.remove(&key);
                }

                if expired_count > 0 {
                    cleanups.fetch_add(expired_count as u64, Ordering::Relaxed);
                    debug!("Cleaned up {} expired cache entries", expired_count);
                }
            }
        });
    }

    /// Evict oldest entries when cache is full
    async fn evict_oldest_entries(&self) {
        let current_size = self.cache.len();
        if current_size <= self.max_entries {
            return; // No eviction needed
        }

        let mut entries_to_remove = Vec::new();

        // Collect entries with their creation times
        for entry in self.cache.iter() {
            entries_to_remove.push((entry.key().clone(), entry.value().created_at));
        }

        // Sort by creation time (oldest first)
        entries_to_remove.sort_by_key(|(_, created_at)| *created_at);

        // Calculate how many entries to evict to get back to 90% of max capacity
        let target_size = (self.max_entries * 9) / 10; // 90% of max
        let entries_to_evict = current_size.saturating_sub(target_size).max(1);

        for (key, _) in entries_to_remove.into_iter().take(entries_to_evict) {
            self.cache.remove(&key);
        }

        debug!(
            "Evicted {} old cache entries (was {}, now {})",
            entries_to_evict,
            current_size,
            self.cache.len()
        );
    }

    /// Warm up cache with common searches (can be called periodically)
    pub async fn warm_cache(
        &self,
        provider: &AnimeProvider,
        common_queries: Vec<&str>,
    ) -> AppResult<()> {
        info!(
            "Warming cache for {} with {} queries",
            provider,
            common_queries.len()
        );

        for query in common_queries {
            // Only warm if not already cached
            if self.get_search_results(provider, query).await.is_none() {
                // This would typically trigger a real API call in the actual implementation
                // For now, we'll just cache an empty result to mark it as "warmed"
                self.cache_search_results(provider, query, vec![]).await?;
            }
        }

        Ok(())
    }
}

impl Default for ProviderCache {
    fn default() -> Self {
        Self::default()
    }
}
