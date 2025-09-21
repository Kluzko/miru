use crate::{
    log_debug,
    modules::provider::traits::{AnimeProviderClient, RateLimiterInfo},
    modules::{
        anime::AnimeDetailed,
        provider::{
            infrastructure::external::{anilist::AniListClient, jikan::JikanClient},
            AnimeProvider, ProviderCache,
        },
    },
    shared::errors::{AppError, AppResult},
    shared::utils::logger::LogContext,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{collections::HashMap, sync::Arc};
use tracing::info;

/// Configuration for a single anime provider (no rate limit duplication)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderConfig {
    /// Display name of the provider
    pub name: String,
    /// Description of what the provider offers
    pub description: String,
    /// API base URL
    pub api_url: String,
    /// Whether the provider is enabled
    pub enabled: bool,
    /// Priority order (lower = higher priority)
    pub priority: u32,
}

// Rate limiting is now handled by individual clients

/// Manages multiple anime providers with configuration and rate limiting
pub struct ProviderManager {
    /// Provider configurations (no rate limit duplication)
    configs: HashMap<AnimeProvider, ProviderConfig>,
    /// Actual client instances implementing the trait
    clients: HashMap<AnimeProvider, Arc<dyn AnimeProviderClient>>,
    /// Primary provider preference
    primary_provider: AnimeProvider,
    /// Shared cache instance for all providers
    cache: Arc<ProviderCache>,
}

impl ProviderManager {
    /// Create a new ProviderManager with default configurations
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        // Default configurations for each provider
        let jikan_config = ProviderConfig {
            name: "MyAnimeList (Jikan)".to_string(),
            description:
                "Comprehensive anime database with ratings, reviews, and detailed metadata"
                    .to_string(),
            api_url: "https://api.jikan.moe/v4".to_string(),

            enabled: true,
            priority: 2,
        };

        let anilist_config = ProviderConfig {
            name: "AniList".to_string(),
            description: "Community-driven database with GraphQL API and modern interface"
                .to_string(),
            api_url: "https://graphql.anilist.co".to_string(),

            enabled: true,
            priority: 1,
        };

        configs.insert(AnimeProvider::Jikan, jikan_config);
        configs.insert(AnimeProvider::AniList, anilist_config);

        // Create shared cache for all providers
        let shared_cache = Arc::new(ProviderCache::new(
            5,    // 5 minutes default TTL
            2,    // 2 minutes TTL for not found results
            2000, // 2000 max entries across all providers
        ));

        info!("Initialized shared provider cache with 5min TTL and 2000 max entries");

        // Initialize clients with shared cache
        let mut clients: HashMap<AnimeProvider, Arc<dyn AnimeProviderClient>> = HashMap::new();

        let jikan_client = Arc::new(
            JikanClient::with_cache(shared_cache.clone())
                .expect("Failed to initialize Jikan client with cache"),
        );
        let anilist_client = Arc::new(
            AniListClient::with_cache(shared_cache.clone())
                .expect("Failed to initialize AniList client with cache"),
        );

        clients.insert(AnimeProvider::Jikan, jikan_client);
        clients.insert(AnimeProvider::AniList, anilist_client);

        Self {
            configs,
            clients,
            primary_provider: AnimeProvider::AniList,
            cache: shared_cache,
        }
    }

    /// Get rate limiter info from actual client (single source of truth)
    pub fn get_provider_rate_limit(&self, provider: &AnimeProvider) -> Option<RateLimiterInfo> {
        self.clients
            .get(provider)
            .map(|client| client.get_rate_limit_info())
    }

    /// Set primary provider
    pub fn set_primary_provider(&mut self, provider: AnimeProvider) -> AppResult<()> {
        if let Some(config) = self.configs.get(&provider) {
            if !config.enabled {
                return Err(AppError::InvalidInput(format!(
                    "Provider {:?} is disabled",
                    provider
                )));
            }
        } else {
            return Err(AppError::InvalidInput(format!(
                "Provider {:?} not found",
                provider
            )));
        }

        self.primary_provider = provider;
        Ok(())
    }

    /// Get primary provider
    pub fn get_primary_provider(&self) -> AnimeProvider {
        self.primary_provider.clone()
    }

    /// Get enabled providers sorted by priority
    pub fn get_enabled_providers(&self) -> Vec<AnimeProvider> {
        let mut providers: Vec<_> = self
            .configs
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(provider, config)| (provider.clone(), config.priority))
            .collect();

        providers.sort_by_key(|(_, priority)| *priority);
        providers
            .into_iter()
            .map(|(provider, _)| provider)
            .collect()
    }

    /// Search for anime using primary provider with fallback
    pub async fn search_anime(
        &mut self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        let enabled_providers = self.get_enabled_providers();

        // Try primary provider first if it's enabled
        let primary_provider = self.primary_provider.clone();
        if enabled_providers.contains(&primary_provider) {
            if let Ok(results) = self
                .search_with_provider(&primary_provider, query, limit)
                .await
            {
                if !results.is_empty() {
                    return Ok(results);
                }
            }
        }

        // Try other enabled providers as fallbacks
        for provider in enabled_providers {
            if provider == self.primary_provider {
                continue; // Already tried
            }

            match self.search_with_provider(&provider, query, limit).await {
                Ok(results) if !results.is_empty() => {
                    log_debug!("Used fallback provider {:?} for search", provider);
                    return Ok(results);
                }
                Ok(_) => continue,
                Err(e) => {
                    LogContext::error_with_context(
                        &e,
                        &format!("Provider {:?} failed for search '{}'", provider, query),
                    );
                    continue;
                }
            }
        }

        Ok(Vec::new())
    }

    /// Search using a specific provider
    async fn search_with_provider(
        &mut self,
        provider: &AnimeProvider,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        // Rate limiting is handled by individual clients

        if let Some(client) = self.clients.get(provider) {
            client.search_anime(query, limit).await
        } else {
            Err(AppError::NotImplemented(format!(
                "Provider {:?} not available",
                provider
            )))
        }
    }

    /// Get anime by ID from specific provider
    pub async fn get_anime_by_id(
        &mut self,
        provider: AnimeProvider,
        id: &str,
    ) -> AppResult<Option<AnimeDetailed>> {
        if let Some(config) = self.configs.get(&provider) {
            if !config.enabled {
                return Err(AppError::InvalidInput(format!(
                    "Provider {:?} is disabled",
                    provider
                )));
            }
        }

        // Rate limiting is handled by individual clients
        if let Some(client) = self.clients.get(&provider) {
            client.get_anime_by_id(id).await
        } else {
            Err(AppError::NotImplemented(format!(
                "Provider {:?} not available",
                provider
            )))
        }
    }

    /// Get top anime from primary provider
    pub async fn get_top_anime(&mut self, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        let provider = self.primary_provider.clone();

        // Rate limiting is handled by individual clients
        if let Some(client) = self.clients.get(&provider) {
            client.get_top_anime(1, limit as i32).await
        } else {
            Err(AppError::NotImplemented(format!(
                "Provider {:?} not available",
                provider
            )))
        }
    }

    /// Get seasonal anime from primary provider
    pub async fn get_seasonal_anime(
        &mut self,
        year: i32,
        season: &str,
        _limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        let provider = self.primary_provider.clone();

        // Rate limiting is handled by individual clients
        if let Some(client) = self.clients.get(&provider) {
            client.get_seasonal_anime(year, season, 1).await
        } else {
            Err(AppError::NotImplemented(format!(
                "Provider {:?} not available",
                provider
            )))
        }
    }

    /// Get comprehensive cache statistics across all providers
    pub async fn get_cache_stats(
        &self,
    ) -> crate::modules::provider::infrastructure::cache::provider_cache::CacheStats {
        self.cache.get_stats().await
    }

    /// Clear all cached data across all providers
    pub async fn clear_all_caches(&self) {
        self.cache.clear().await;
        info!("All provider caches cleared via ProviderManager");
    }

    /// Warm up cache with common search terms
    pub async fn warm_cache(&self, common_queries: Vec<&str>) -> AppResult<()> {
        info!("Warming cache with {} common queries", common_queries.len());

        for provider in [AnimeProvider::Jikan, AnimeProvider::AniList] {
            if let Some(config) = self.configs.get(&provider) {
                if config.enabled {
                    if let Err(e) = self
                        .cache
                        .warm_cache(&provider, common_queries.clone())
                        .await
                    {
                        LogContext::error_with_context(
                            &e,
                            &format!("Failed to warm cache for provider {:?}", provider),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if providers are sharing the cache correctly (for debugging)
    pub async fn validate_shared_cache(&self) -> bool {
        // This method helps verify that all providers are using the same cache instance
        let stats_before = self.cache.get_stats().await;

        // Try to cache a test entry through one provider
        let _ = self
            .cache
            .cache_search_results(&AnimeProvider::Jikan, "test_shared_cache", vec![])
            .await;

        // Check if the cache was updated
        let stats_after = self.cache.get_stats().await;

        stats_after.entries_count > stats_before.entries_count
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}
