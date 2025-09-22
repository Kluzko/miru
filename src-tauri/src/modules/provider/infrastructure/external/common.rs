use crate::modules::anime::AnimeDetailed;
use crate::modules::provider::{AnimeProvider, ProviderCache};
use crate::shared::errors::AppResult;
use std::sync::Arc;
use tracing::{debug, warn};

/// Common caching behavior for all provider clients
/// Eliminates duplicate caching logic across Jikan and AniList
pub struct CachedProviderBehavior {
    cache: Arc<ProviderCache>,
    provider_type: AnimeProvider,
}

impl CachedProviderBehavior {
    pub fn new(cache: Arc<ProviderCache>, provider_type: AnimeProvider) -> Self {
        Self {
            cache,
            provider_type,
        }
    }

    /// Execute a search with automatic caching
    pub async fn cached_search<F, Fut>(
        &self,
        query: &str,
        limit: usize,
        search_fn: F,
    ) -> AppResult<Vec<AnimeDetailed>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = AppResult<Vec<AnimeDetailed>>>,
    {
        if query.trim().is_empty() {
            return Err(crate::shared::errors::AppError::ValidationError(
                "Search query cannot be empty".to_string(),
            ));
        }

        // Create cache key that includes limit to ensure proper caching
        let cache_query = format!("{}:limit:{}", query.trim(), limit);

        // Check cache first
        if let Some(cached_results) = self
            .cache
            .get_search_results(&self.provider_type, &cache_query)
            .await
        {
            debug!(
                "{:?} search cache hit for query: {}",
                self.provider_type, query
            );
            return Ok(cached_results);
        }

        // Check if request is already in progress to prevent duplicate concurrent requests
        if self
            .cache
            .is_request_in_progress(&self.provider_type, &cache_query)
            .await
        {
            debug!(
                "{:?} search already in progress for query: {}, waiting for result",
                self.provider_type, query
            );

            // Wait a bit and check cache again
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if let Some(cached_results) = self
                .cache
                .get_search_results(&self.provider_type, &cache_query)
                .await
            {
                return Ok(cached_results);
            }

            // If still not available, proceed with the request (fallback)
            warn!(
                "{:?} search in progress but no cached result found, proceeding with API call",
                self.provider_type
            );
        }

        // Mark request as in progress
        self.cache
            .mark_request_in_progress(&self.provider_type, &cache_query)
            .await;

        // Perform API call
        let api_result = search_fn().await;

        // Remove in-progress marker
        self.cache
            .remove_request_in_progress(&self.provider_type, &cache_query)
            .await;

        match api_result {
            Ok(results) => {
                // Cache successful results
                if let Err(e) = self
                    .cache
                    .cache_search_results(&self.provider_type, &cache_query, results.clone())
                    .await
                {
                    warn!(
                        "Failed to cache {:?} search results: {}",
                        self.provider_type, e
                    );
                }
                debug!(
                    "{:?} search API call completed and cached for query: {}",
                    self.provider_type, query
                );
                Ok(results)
            }
            Err(e) => {
                // Don't cache error responses, just return the error
                debug!(
                    "{:?} search API call failed for query: {}, error: {}",
                    self.provider_type, query, e
                );
                Err(e)
            }
        }
    }

    /// Get cache statistics for monitoring
    pub async fn get_cache_stats(
        &self,
    ) -> crate::modules::provider::infrastructure::cache::provider_cache::CacheStats {
        self.cache.get_stats().await
    }

    /// Clear all cached data
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
        debug!("{:?} client cache cleared", self.provider_type);
    }
}
