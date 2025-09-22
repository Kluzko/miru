use crate::modules::provider::{
    infrastructure::{
        external::{anilist::AniListClient, jikan::JikanClient},
        service::provider_service::ProviderConfig,
    },
    traits::AnimeProviderClient,
    AnimeProvider, ProviderCache,
};
use crate::shared::errors::AppResult;
use std::sync::Arc;

/// Factory trait for creating provider instances
pub trait ProviderFactory: Send + Sync {
    /// Create a provider instance with the given configuration and cache
    fn create_provider(
        &self,
        config: &ProviderConfig,
        cache: Arc<ProviderCache>,
    ) -> AppResult<Arc<dyn AnimeProviderClient>>;

    /// Get the provider type this factory creates
    fn provider_type(&self) -> AnimeProvider;

    /// Get default configuration for this provider
    fn default_config(&self) -> ProviderConfig;

    /// Check if this factory can create the requested provider type
    fn supports(&self, provider_type: &AnimeProvider) -> bool {
        &self.provider_type() == provider_type
    }
}

/// Factory for creating Jikan clients
pub struct JikanProviderFactory;

impl ProviderFactory for JikanProviderFactory {
    fn create_provider(
        &self,
        _config: &ProviderConfig,
        cache: Arc<ProviderCache>,
    ) -> AppResult<Arc<dyn AnimeProviderClient>> {
        let client = JikanClient::with_cache(cache)?;
        Ok(Arc::new(client))
    }

    fn provider_type(&self) -> AnimeProvider {
        AnimeProvider::Jikan
    }

    fn default_config(&self) -> ProviderConfig {
        ProviderConfig {
            name: "MyAnimeList (Jikan)".to_string(),
            description:
                "Comprehensive anime database with ratings, reviews, and detailed metadata"
                    .to_string(),
            api_url: "https://api.jikan.moe/v4".to_string(),
            enabled: true,
            priority: 2,
        }
    }
}

/// Factory for creating AniList clients
pub struct AniListProviderFactory;

impl ProviderFactory for AniListProviderFactory {
    fn create_provider(
        &self,
        _config: &ProviderConfig,
        cache: Arc<ProviderCache>,
    ) -> AppResult<Arc<dyn AnimeProviderClient>> {
        let client = AniListClient::with_cache(cache)?;
        Ok(Arc::new(client))
    }

    fn provider_type(&self) -> AnimeProvider {
        AnimeProvider::AniList
    }

    fn default_config(&self) -> ProviderConfig {
        ProviderConfig {
            name: "AniList".to_string(),
            description: "Community-driven database with GraphQL API and modern interface"
                .to_string(),
            api_url: "https://graphql.anilist.co".to_string(),
            enabled: true,
            priority: 1,
        }
    }
}

/// Factory for creating provider instances based on their type
/// This is the main factory that delegates to specific provider factories
pub struct ProviderFactoryManager {
    factories: Vec<Box<dyn ProviderFactory>>,
}

impl ProviderFactoryManager {
    /// Create a new factory manager with default factories
    pub fn new() -> Self {
        let mut factories: Vec<Box<dyn ProviderFactory>> = Vec::new();

        // Register existing provider factories
        factories.push(Box::new(JikanProviderFactory));
        factories.push(Box::new(AniListProviderFactory));

        Self { factories }
    }

    /// Create a provider instance for the given type
    pub fn create_provider(
        &self,
        provider_type: &AnimeProvider,
        config: &ProviderConfig,
        cache: Arc<ProviderCache>,
    ) -> AppResult<Arc<dyn AnimeProviderClient>> {
        for factory in &self.factories {
            if factory.supports(provider_type) {
                return factory.create_provider(config, cache);
            }
        }

        Err(crate::shared::errors::AppError::NotImplemented(format!(
            "No factory found for provider: {:?}",
            provider_type
        )))
    }

    /// Get default configuration for a provider type
    pub fn get_default_config(&self, provider_type: &AnimeProvider) -> Option<ProviderConfig> {
        for factory in &self.factories {
            if factory.supports(provider_type) {
                return Some(factory.default_config());
            }
        }
        None
    }

    /// Get all supported provider types
    pub fn get_supported_providers(&self) -> Vec<AnimeProvider> {
        self.factories.iter().map(|f| f.provider_type()).collect()
    }

    /// Register a new provider factory (for future extensibility)
    #[allow(dead_code)]
    pub fn register_factory(&mut self, factory: Box<dyn ProviderFactory>) {
        self.factories.push(factory);
    }
}

impl Default for ProviderFactoryManager {
    fn default() -> Self {
        Self::new()
    }
}
