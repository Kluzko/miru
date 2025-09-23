use super::aggregation::AnimeDataAggregator;
use crate::{
    log_debug,
    modules::anime::AnimeDetailed,
    modules::provider::{
        domain::services::unified_registry::{UnifiedProviderRegistry, UnifiedProviderStats},
        infrastructure::cache::ProviderCache,
        traits::{AnimeProviderClient, RateLimiterInfo},
        AnimeProvider, ProviderFactoryManager,
    },
    shared::errors::{AppError, AppResult},
    shared::utils::logger::LogContext,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::RwLock;
use tracing::info;

/// Configuration for a single anime provider (immutable after initialization)
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

/// Optimized provider service for Tauri applications
/// Uses unified registry, streaming aggregation, and optimized cache for better performance
pub struct ProviderService {
    /// Factory manager for creating provider instances
    factory_manager: ProviderFactoryManager,
    /// Thread-safe client instances
    clients: HashMap<AnimeProvider, Arc<dyn AnimeProviderClient>>,
    /// Unified provider registry with integrated health monitoring
    registry: Arc<RwLock<UnifiedProviderRegistry>>,
    /// Shared cache instance for all providers
    cache: Arc<ProviderCache>,
}

impl ProviderService {
    /// Create a new ProviderService using factory pattern and registry
    pub fn new() -> Self {
        // Create shared cache for all providers
        let shared_cache = Arc::new(ProviderCache::new(
            5,    // 5 minutes default TTL
            2,    // 2 minutes TTL for not found results
            2000, // 2000 max entries across all providers
        ));

        info!("Initialized shared provider cache with 5min TTL and 2000 max entries");

        // Initialize factory manager and registry
        let factory_manager = ProviderFactoryManager::new();
        let registry = Arc::new(RwLock::new(UnifiedProviderRegistry::new()));

        // Initialize clients using factory pattern
        let mut clients: HashMap<AnimeProvider, Arc<dyn AnimeProviderClient>> = HashMap::new();

        // Get all supported providers from factory manager
        for provider_type in factory_manager.get_supported_providers() {
            if let Some(config) = factory_manager.get_default_config(&provider_type) {
                match factory_manager.create_provider(&provider_type, &config, shared_cache.clone())
                {
                    Ok(client) => {
                        clients.insert(provider_type.clone(), client);
                        info!("Successfully initialized provider: {:?}", provider_type);
                    }
                    Err(e) => {
                        // Log error but continue with other providers
                        LogContext::error_with_context(
                            &e,
                            &format!("Failed to initialize provider: {:?}", provider_type),
                        );
                    }
                }
            }
        }

        info!(
            "ProviderService initialized with {} providers",
            clients.len()
        );

        Self {
            factory_manager,
            clients,
            registry,
            cache: shared_cache,
        }
    }

    /// Get rate limiter info from actual client (lock-free read)
    pub fn get_provider_rate_limit(&self, provider: &AnimeProvider) -> Option<RateLimiterInfo> {
        self.clients
            .get(provider)
            .map(|client| client.get_rate_limit_info())
    }

    /// Set primary provider using registry
    pub async fn set_primary_provider(&self, provider: AnimeProvider) -> AppResult<()> {
        let mut registry = self.registry.write().await;
        registry
            .set_primary_provider(provider)
            .map_err(|e| AppError::InvalidInput(e))
    }

    /// Get primary provider from registry
    pub async fn get_primary_provider(&self) -> AnimeProvider {
        let registry = self.registry.read().await;
        registry.get_primary_provider()
    }

    /// Get enabled providers sorted by priority and health
    pub async fn get_enabled_providers(&self) -> Vec<AnimeProvider> {
        let registry = self.registry.read().await;
        registry.get_enabled_providers()
    }

    /// Search for anime using primary provider with fallback and health monitoring
    pub async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        let enabled_providers = self.get_enabled_providers().await;

        // Try primary provider first if it's enabled
        let primary_provider = self.get_primary_provider().await;
        if enabled_providers.contains(&primary_provider) {
            let start_time = Instant::now();
            match self
                .search_with_provider(&primary_provider, query, limit)
                .await
            {
                Ok(results) if !results.is_empty() => {
                    // Record successful request with response time
                    let mut registry = self.registry.write().await;
                    registry.record_success(&primary_provider, start_time.elapsed());
                    return Ok(results);
                }
                Ok(_) => {
                    // Empty results still count as success
                    let mut registry = self.registry.write().await;
                    registry.record_success(&primary_provider, start_time.elapsed());
                }
                Err(e) => {
                    // Record failure
                    let mut registry = self.registry.write().await;
                    registry.record_failure(&primary_provider);
                    LogContext::error_with_context(
                        &e,
                        &format!(
                            "Primary provider {:?} failed for search '{}'",
                            primary_provider, query
                        ),
                    );
                }
            }
        }

        // Try other enabled providers as fallbacks
        for provider in enabled_providers {
            if provider == primary_provider {
                continue; // Already tried
            }

            let start_time = Instant::now();
            match self.search_with_provider(&provider, query, limit).await {
                Ok(results) if !results.is_empty() => {
                    log_debug!("Used fallback provider {:?} for search", provider);
                    // Record successful fallback
                    let mut registry = self.registry.write().await;
                    registry.record_success(&provider, start_time.elapsed());
                    return Ok(results);
                }
                Ok(_) => {
                    // Empty results still count as success
                    let mut registry = self.registry.write().await;
                    registry.record_success(&provider, start_time.elapsed());
                }
                Err(e) => {
                    // Record failure and continue to next provider
                    let mut registry = self.registry.write().await;
                    registry.record_failure(&provider);
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

    /// Comprehensive search that aggregates data from multiple providers
    /// This is the recommended method for getting the most complete anime data
    pub async fn search_anime_comprehensive(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        let enabled_providers = self.get_enabled_providers().await;
        let primary_provider = self.get_primary_provider().await;

        if enabled_providers.is_empty() {
            return Err(AppError::InvalidInput(
                "No providers are enabled".to_string(),
            ));
        }

        let mut provider_results = HashMap::new();

        // Search all enabled providers concurrently
        let mut search_tasks = Vec::new();
        for provider in &enabled_providers {
            let provider_clone = provider.clone();
            let query_clone = query.to_string();

            if let Some(client) = self.clients.get(&provider) {
                let client_clone = client.clone();
                let registry_clone = self.registry.clone();

                let task = tokio::spawn(async move {
                    let start_time = Instant::now();
                    match client_clone.search_anime(&query_clone, limit).await {
                        Ok(results) => {
                            // Record success
                            {
                                let mut registry = registry_clone.write().await;
                                registry.record_success(&provider_clone, start_time.elapsed());
                            }
                            Some((provider_clone, results))
                        }
                        Err(e) => {
                            // Record failure
                            {
                                let mut registry = registry_clone.write().await;
                                registry.record_failure(&provider_clone);
                            }
                            LogContext::error_with_context(
                                &e,
                                &format!(
                                    "Provider {:?} failed for comprehensive search '{}'",
                                    provider_clone, query_clone
                                ),
                            );
                            None
                        }
                    }
                });
                search_tasks.push(task);
            }
        }

        // Wait for all searches to complete
        for task in search_tasks {
            if let Ok(Some((provider, results))) = task.await {
                if !results.is_empty() {
                    provider_results.insert(provider, results);
                }
            }
        }

        if provider_results.is_empty() {
            return Ok(Vec::new());
        }

        // Use intelligent aggregation to merge and enhance data from multiple providers
        // This fills gaps in data and provides more reliable information by cross-referencing
        let comprehensive_results = AnimeDataAggregator::create_comprehensive_search_results(
            provider_results,
            primary_provider,
            limit,
        );

        log_debug!(
            "Comprehensive search for '{}' returned {} results from {} providers",
            query,
            comprehensive_results.len(),
            enabled_providers.len()
        );

        Ok(comprehensive_results)
    }

    /// Get anime by ID with comprehensive data from multiple providers
    /// This fetches from multiple providers and merges data to fill gaps and verify accuracy
    pub async fn get_anime_by_id_comprehensive(
        &self,
        id: &str,
        preferred_provider: Option<AnimeProvider>,
    ) -> AppResult<Option<AnimeDetailed>> {
        let enabled_providers = self.get_enabled_providers().await;
        let primary_provider =
            preferred_provider.unwrap_or_else(|| self.get_primary_provider_sync());

        if enabled_providers.is_empty() {
            return Err(AppError::InvalidInput(
                "No providers are enabled".to_string(),
            ));
        }

        let mut provider_results = HashMap::new();
        let mut search_tasks = Vec::new();

        // Fetch from all enabled providers concurrently
        for provider in &enabled_providers {
            let provider_clone = provider.clone();
            let id_clone = id.to_string();

            if let Some(client) = self.clients.get(&provider) {
                let client_clone = client.clone();
                let registry_clone = self.registry.clone();

                let task = tokio::spawn(async move {
                    let start_time = Instant::now();
                    match client_clone.get_anime_by_id(&id_clone).await {
                        Ok(Some(anime)) => {
                            let mut registry = registry_clone.write().await;
                            registry.record_success(&provider_clone, start_time.elapsed());
                            Some((provider_clone, anime))
                        }
                        Ok(None) => {
                            let mut registry = registry_clone.write().await;
                            registry.record_success(&provider_clone, start_time.elapsed());
                            None
                        }
                        Err(e) => {
                            let mut registry = registry_clone.write().await;
                            registry.record_failure(&provider_clone);
                            LogContext::error_with_context(
                                &e,
                                &format!(
                                    "Provider {:?} failed for get_by_id '{}'",
                                    provider_clone, id_clone
                                ),
                            );
                            None
                        }
                    }
                });
                search_tasks.push(task);
            }
        }

        // Collect results from all providers
        for task in search_tasks {
            if let Ok(Some((provider, anime))) = task.await {
                provider_results.insert(provider, vec![anime]);
            }
        }

        if provider_results.is_empty() {
            return Ok(None);
        }

        // Use aggregation to merge data from multiple providers for the most complete anime info
        let merged_results =
            AnimeDataAggregator::merge_anime_data(provider_results, primary_provider);

        // Return the first (and should be only) result as it's aggregated from multiple sources
        Ok(merged_results.into_iter().next())
    }

    /// Get anime by ID with fallback (legacy method - use get_anime_by_id_comprehensive for best results)
    pub async fn get_anime_by_id_with_fallback(
        &self,
        id: &str,
        preferred_provider: Option<AnimeProvider>,
    ) -> AppResult<Option<AnimeDetailed>> {
        let target_provider = preferred_provider.unwrap_or_else(|| {
            // Try to determine provider from ID format
            if id.parse::<i32>().is_ok() {
                // Numeric ID could be either Jikan (MAL) or AniList
                // Try both, but prefer primary provider
                self.get_primary_provider_sync()
            } else {
                // Non-numeric ID, use primary provider
                self.get_primary_provider_sync()
            }
        });

        // Try primary/preferred provider first
        {
            let registry = self.registry.read().await;
            if registry.is_provider_available(&target_provider) {
                drop(registry); // Release lock before async call

                let start_time = Instant::now();
                match self
                    .get_anime_by_id_from_provider(&target_provider, id)
                    .await
                {
                    Ok(Some(anime)) => {
                        let mut registry = self.registry.write().await;
                        registry.record_success(&target_provider, start_time.elapsed());
                        return Ok(Some(anime));
                    }
                    Ok(None) => {
                        let mut registry = self.registry.write().await;
                        registry.record_success(&target_provider, start_time.elapsed());
                        // Continue to try other providers
                    }
                    Err(e) => {
                        let mut registry = self.registry.write().await;
                        registry.record_failure(&target_provider);
                        LogContext::error_with_context(
                            &e,
                            &format!(
                                "Provider {:?} failed for get_by_id '{}'",
                                target_provider, id
                            ),
                        );
                        // Continue to try other providers
                    }
                }
            }
        }

        // Try other providers as fallback
        let enabled_providers = self.get_enabled_providers().await;
        for provider in enabled_providers {
            if provider == target_provider {
                continue; // Already tried
            }

            let start_time = Instant::now();
            match self.get_anime_by_id_from_provider(&provider, id).await {
                Ok(Some(anime)) => {
                    log_debug!("Used fallback provider {:?} for get_by_id", provider);
                    let mut registry = self.registry.write().await;
                    registry.record_success(&provider, start_time.elapsed());
                    return Ok(Some(anime));
                }
                Ok(None) => {
                    let mut registry = self.registry.write().await;
                    registry.record_success(&provider, start_time.elapsed());
                    continue;
                }
                Err(e) => {
                    let mut registry = self.registry.write().await;
                    registry.record_failure(&provider);
                    LogContext::error_with_context(
                        &e,
                        &format!("Provider {:?} failed for get_by_id '{}'", provider, id),
                    );
                    continue;
                }
            }
        }

        Ok(None)
    }

    /// Helper to get anime by ID from a specific provider
    async fn get_anime_by_id_from_provider(
        &self,
        provider: &AnimeProvider,
        id: &str,
    ) -> AppResult<Option<AnimeDetailed>> {
        if let Some(client) = self.clients.get(provider) {
            client.get_anime_by_id(id).await
        } else {
            Err(AppError::NotImplemented(format!(
                "Provider {:?} not available",
                provider
            )))
        }
    }

    /// Sync version of get_primary_provider for use in sync contexts
    fn get_primary_provider_sync(&self) -> AnimeProvider {
        // This is a simplified version that returns a default
        // In real implementation, you might want to use a cached value
        AnimeProvider::default()
    }
    /// Search using a specific provider (lock-free)
    async fn search_with_provider(
        &self,
        provider: &AnimeProvider,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        if let Some(client) = self.clients.get(provider) {
            client.search_anime(query, limit).await
        } else {
            Err(AppError::NotImplemented(format!(
                "Provider {:?} not available",
                provider
            )))
        }
    }

    /// Get anime by ID from specific provider with health monitoring
    pub async fn get_anime_by_id(
        &self,
        provider: AnimeProvider,
        id: &str,
    ) -> AppResult<Option<AnimeDetailed>> {
        // Check if provider is available through registry
        {
            let registry = self.registry.read().await;
            if !registry.is_provider_available(&provider) {
                return Err(AppError::InvalidInput(format!(
                    "Provider {:?} is not available",
                    provider
                )));
            }
        }

        let start_time = Instant::now();
        match self.clients.get(&provider) {
            Some(client) => {
                match client.get_anime_by_id(id).await {
                    Ok(result) => {
                        // Record successful request
                        let mut registry = self.registry.write().await;
                        registry.record_success(&provider, start_time.elapsed());
                        Ok(result)
                    }
                    Err(e) => {
                        // Record failure
                        let mut registry = self.registry.write().await;
                        registry.record_failure(&provider);
                        Err(e)
                    }
                }
            }
            None => Err(AppError::NotImplemented(format!(
                "Provider {:?} not available",
                provider
            ))),
        }
    }

    /// Get top anime from primary provider with health monitoring
    pub async fn get_top_anime(&self, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        let provider = self.get_primary_provider().await;
        let start_time = Instant::now();

        match self.clients.get(&provider) {
            Some(client) => {
                match client.get_top_anime(1, limit as i32).await {
                    Ok(results) => {
                        // Record successful request
                        let mut registry = self.registry.write().await;
                        registry.record_success(&provider, start_time.elapsed());
                        Ok(results)
                    }
                    Err(e) => {
                        // Record failure
                        let mut registry = self.registry.write().await;
                        registry.record_failure(&provider);
                        Err(e)
                    }
                }
            }
            None => Err(AppError::NotImplemented(format!(
                "Provider {:?} not available",
                provider
            ))),
        }
    }

    /// Get seasonal anime from primary provider with health monitoring
    pub async fn get_seasonal_anime(
        &self,
        year: i32,
        season: &str,
        _limit: usize,
    ) -> AppResult<Vec<AnimeDetailed>> {
        let provider = self.get_primary_provider().await;
        let start_time = Instant::now();

        match self.clients.get(&provider) {
            Some(client) => {
                match client.get_seasonal_anime(year, season, 1).await {
                    Ok(results) => {
                        // Record successful request
                        let mut registry = self.registry.write().await;
                        registry.record_success(&provider, start_time.elapsed());
                        Ok(results)
                    }
                    Err(e) => {
                        // Record failure
                        let mut registry = self.registry.write().await;
                        registry.record_failure(&provider);
                        Err(e)
                    }
                }
            }
            None => Err(AppError::NotImplemented(format!(
                "Provider {:?} not available",
                provider
            ))),
        }
    }

    /// Get comprehensive cache statistics across all providers
    pub async fn get_cache_stats(
        &self,
    ) -> crate::modules::provider::infrastructure::cache::provider_cache::CacheStats {
        self.cache.get_stats().await
    }

    /// Clear all cached data across all providers (lock-free)
    pub async fn clear_all_caches(&self) {
        self.cache.clear().await;
        info!("All provider caches cleared via ProviderService");
    }

    /// Warm up cache with common search terms using available providers
    pub async fn warm_cache(&self, common_queries: Vec<&str>) -> AppResult<()> {
        info!("Warming cache with {} common queries", common_queries.len());

        let enabled_providers = self.get_enabled_providers().await;
        for provider in enabled_providers {
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

        Ok(())
    }

    /// Get provider health status for monitoring
    pub async fn get_provider_health(
        &self,
        provider: &AnimeProvider,
    ) -> Option<UnifiedProviderStats> {
        let registry = self.registry.read().await;
        registry.get_stats_summary().get(provider).cloned()
    }

    /// Get all provider health statuses for monitoring dashboard
    pub async fn get_all_provider_health(&self) -> HashMap<AnimeProvider, UnifiedProviderStats> {
        let registry = self.registry.read().await;
        registry.get_stats_summary()
    }

    /// Get list of currently available (healthy) providers
    pub async fn get_available_providers(&self) -> Vec<AnimeProvider> {
        let registry = self.registry.read().await;
        registry.get_available_providers()
    }

    /// Check if providers are sharing the cache correctly (lock-free debugging)
    pub async fn validate_shared_cache(&self) -> bool {
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

impl Default for ProviderService {
    fn default() -> Self {
        Self::new()
    }
}
