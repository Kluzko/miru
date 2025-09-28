use super::provider_enum::AnimeProvider;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

/// Provider-specific metadata for external IDs and synchronization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ProviderMetadata {
    /// External IDs from different providers
    pub external_ids: HashMap<AnimeProvider, String>,

    /// URLs to provider pages
    pub provider_urls: HashMap<AnimeProvider, String>,

    /// User's preferred provider for primary data
    pub user_preferred_provider: Option<AnimeProvider>,

    /// Current primary provider (can be different from user preference if not available)
    pub primary_provider: AnimeProvider,
}

impl ProviderMetadata {
    pub fn new(primary_provider: AnimeProvider, external_id: String) -> Self {
        let mut external_ids = HashMap::new();
        external_ids.insert(primary_provider.clone(), external_id);

        Self {
            external_ids,
            provider_urls: HashMap::new(),
            user_preferred_provider: None,
            primary_provider,
        }
    }

    /// Get external ID for specific provider
    pub fn get_external_id(&self, provider: &AnimeProvider) -> Option<&String> {
        self.external_ids.get(provider)
    }

    /// Add external ID for specific provider
    pub fn add_external_id(&mut self, provider: AnimeProvider, id: String) {
        self.external_ids.insert(provider, id);
    }

    /// Get URL for specific provider
    pub fn get_provider_url(&self, provider: &AnimeProvider) -> Option<&String> {
        self.provider_urls.get(provider)
    }

    /// Add provider URL
    pub fn add_provider_url(&mut self, provider: AnimeProvider, url: String) {
        self.provider_urls.insert(provider, url);
    }

    /// Check if provider is available
    pub fn has_provider(&self, provider: &AnimeProvider) -> bool {
        self.external_ids.contains_key(provider)
    }

    /// Set primary provider
    pub fn set_primary_provider(&mut self, provider: AnimeProvider) -> Result<(), String> {
        if !self.has_provider(&provider) {
            return Err(format!(
                "Provider {:?} not available for this anime",
                provider
            ));
        }

        self.primary_provider = provider.clone();
        self.user_preferred_provider = Some(provider);
        Ok(())
    }
}

impl Default for ProviderMetadata {
    fn default() -> Self {
        Self {
            external_ids: HashMap::new(),
            provider_urls: HashMap::new(),
            user_preferred_provider: None,
            primary_provider: AnimeProvider::default(),
        }
    }
}
