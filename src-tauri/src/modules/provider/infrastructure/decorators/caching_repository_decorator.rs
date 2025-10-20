use async_trait::async_trait;
use std::sync::Arc;

use crate::modules::provider::domain::{
    entities::AnimeData,
    repositories::{AnimeProviderRepository, CacheRepository},
};
use crate::shared::{domain::value_objects::AnimeProvider, errors::AppResult};

/// Decorator that adds transparent caching to any AnimeProviderRepository
///
/// This implements the Decorator Pattern to separate caching concerns from business logic.
/// The decorator wraps any AnimeProviderRepository implementation and automatically
/// handles cache lookups and updates without the business logic needing to know about caching.
///
/// # Design Pattern: Decorator
/// - Component: AnimeProviderRepository trait
/// - ConcreteComponent: ProviderRepositoryAdapter
/// - Decorator: CachingRepositoryDecorator
///
/// # Benefits:
/// - Caching is transparent to business logic
/// - Can be added/removed without changing services
/// - Follows Single Responsibility Principle
/// - Easy to test (mock cache, mock inner repo)
pub struct CachingRepositoryDecorator {
    /// The wrapped repository implementation
    inner: Arc<dyn AnimeProviderRepository>,
    /// The cache implementation
    cache: Arc<dyn CacheRepository>,
}

impl CachingRepositoryDecorator {
    /// Create a new caching decorator
    ///
    /// # Arguments
    /// * `inner` - The repository to wrap with caching
    /// * `cache` - The cache implementation to use
    pub fn new(inner: Arc<dyn AnimeProviderRepository>, cache: Arc<dyn CacheRepository>) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl AnimeProviderRepository for CachingRepositoryDecorator {
    async fn search_anime(
        &self,
        query: &str,
        limit: usize,
        provider: AnimeProvider,
    ) -> AppResult<Vec<AnimeData>> {
        // Check cache first
        if let Some(cached) = self.cache.get_search_results(query, provider).await {
            log::debug!("Cache HIT for search: {} ({:?})", query, provider);
            return Ok(cached);
        }

        log::debug!("Cache MISS for search: {} ({:?})", query, provider);

        // Fetch from inner repository
        let results = self.inner.search_anime(query, limit, provider).await?;

        // Cache results (fire and forget - don't block on cache writes)
        self.cache
            .cache_search_results(query, provider, results.clone())
            .await;

        Ok(results)
    }

    async fn get_anime_by_id(
        &self,
        id: &str,
        provider: AnimeProvider,
    ) -> AppResult<Option<AnimeData>> {
        // Check cache
        if let Some(cached) = self.cache.get_anime_details(id, provider).await {
            log::debug!("Cache HIT for details: {} ({:?})", id, provider);
            return Ok(Some(cached));
        }

        log::debug!("Cache MISS for details: {} ({:?})", id, provider);

        // Fetch from inner repository
        let result = self.inner.get_anime_by_id(id, provider).await?;

        // Cache if found
        if let Some(ref anime_data) = result {
            self.cache
                .cache_anime_details(id, provider, anime_data.clone())
                .await;
        }

        Ok(result)
    }

    async fn is_provider_available(&self, provider: &AnimeProvider) -> bool {
        // Delegate to inner - no caching for availability checks
        // (we want fresh health status)
        self.inner.is_provider_available(provider).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::{
        anime::domain::entities::anime_detailed::AnimeDetailed,
        provider::domain::{
            entities::{AnimeData, DataQuality, DataSource},
            repositories::CacheStats,
        },
    };
    use std::sync::Mutex;

    // Mock implementations for testing

    struct MockInnerRepository {
        search_called: Arc<Mutex<usize>>,
        details_called: Arc<Mutex<usize>>,
    }

    impl MockInnerRepository {
        fn new() -> Self {
            Self {
                search_called: Arc::new(Mutex::new(0)),
                details_called: Arc::new(Mutex::new(0)),
            }
        }
    }

    #[async_trait]
    impl AnimeProviderRepository for MockInnerRepository {
        async fn search_anime(
            &self,
            _query: &str,
            _limit: usize,
            _provider: AnimeProvider,
        ) -> AppResult<Vec<AnimeData>> {
            *self.search_called.lock().unwrap() += 1;
            Ok(vec![]) // Return empty for tests
        }

        async fn get_anime_by_id(
            &self,
            _id: &str,
            _provider: AnimeProvider,
        ) -> AppResult<Option<AnimeData>> {
            *self.details_called.lock().unwrap() += 1;
            Ok(None)
        }

        async fn is_provider_available(&self, _provider: &AnimeProvider) -> bool {
            true
        }
    }

    struct MockCache {
        search_results: Arc<Mutex<Option<Vec<AnimeData>>>>,
        details_results: Arc<Mutex<Option<AnimeData>>>,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                search_results: Arc::new(Mutex::new(None)),
                details_results: Arc::new(Mutex::new(None)),
            }
        }

        fn set_search_cache(&self, data: Vec<AnimeData>) {
            *self.search_results.lock().unwrap() = Some(data);
        }

        fn set_details_cache(&self, data: AnimeData) {
            *self.details_results.lock().unwrap() = Some(data);
        }
    }

    #[async_trait]
    impl CacheRepository for MockCache {
        async fn get_search_results(
            &self,
            _query: &str,
            _provider: AnimeProvider,
        ) -> Option<Vec<AnimeData>> {
            self.search_results.lock().unwrap().clone()
        }

        async fn cache_search_results(
            &self,
            _query: &str,
            _provider: AnimeProvider,
            results: Vec<AnimeData>,
        ) {
            *self.search_results.lock().unwrap() = Some(results);
        }

        async fn get_anime_details(
            &self,
            _id: &str,
            _provider: AnimeProvider,
        ) -> Option<AnimeData> {
            self.details_results.lock().unwrap().clone()
        }

        async fn cache_anime_details(&self, _id: &str, _provider: AnimeProvider, anime: AnimeData) {
            *self.details_results.lock().unwrap() = Some(anime);
        }

        async fn clear_cache(&self) {
            *self.search_results.lock().unwrap() = None;
            *self.details_results.lock().unwrap() = None;
        }

        async fn get_cache_stats(&self) -> CacheStats {
            CacheStats {
                search_entries: 0,
                details_entries: 0,
                total_entries: 0,
                hit_rate: 0.0,
                miss_rate: 0.0,
                search_ttl_seconds: 300,
                details_ttl_seconds: 3600,
            }
        }
    }

    #[tokio::test]
    async fn test_cache_hit_does_not_call_inner_repository() {
        // Arrange
        let inner = Arc::new(MockInnerRepository::new());
        let cache = Arc::new(MockCache::new());

        // Populate cache
        cache.set_search_cache(vec![]);

        let decorator = CachingRepositoryDecorator::new(inner.clone(), cache);

        // Act
        let _ = decorator
            .search_anime("test", 10, AnimeProvider::AniList)
            .await;

        // Assert - inner repository should NOT be called
        assert_eq!(*inner.search_called.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_cache_miss_calls_inner_repository() {
        // Arrange
        let inner = Arc::new(MockInnerRepository::new());
        let cache = Arc::new(MockCache::new());

        let decorator = CachingRepositoryDecorator::new(inner.clone(), cache);

        // Act
        let _ = decorator
            .search_anime("test", 10, AnimeProvider::AniList)
            .await;

        // Assert - inner repository SHOULD be called
        assert_eq!(*inner.search_called.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_availability_check_bypasses_cache() {
        // Arrange
        let inner = Arc::new(MockInnerRepository::new());
        let cache = Arc::new(MockCache::new());

        let decorator = CachingRepositoryDecorator::new(inner, cache);

        // Act
        let available = decorator
            .is_provider_available(&AnimeProvider::AniList)
            .await;

        // Assert - should delegate to inner
        assert!(available);
    }
}
