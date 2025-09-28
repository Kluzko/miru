use crate::modules::anime::domain::services::data_quality_service::DataQualityService;
use crate::modules::anime::AnimeDetailed;
use crate::modules::provider::domain::repositories::{
    anime_provider_repo::AnimeProviderRepository, cache_repo::CacheRepository,
};
use crate::modules::provider::domain::services::{AnimeSearchService, ProviderSelectionService};
use crate::modules::provider::domain::value_objects::{
    provider_enum::AnimeProvider, SearchCriteria,
};
use crate::shared::errors::AppResult;
use std::sync::Arc;

/// Clean application service for provider operations
#[derive(Clone)]
pub struct ProviderService {
    anime_search_service: Arc<AnimeSearchService>,
    data_quality_service: Arc<DataQualityService>,
    provider_selection_service: Arc<ProviderSelectionService>,
}

impl ProviderService {
    pub fn new(
        provider_repository: Arc<dyn AnimeProviderRepository>,
        cache_repository: Arc<dyn CacheRepository>,
    ) -> Self {
        let data_quality_service = Arc::new(DataQualityService::new());
        let provider_selection_service = Arc::new(ProviderSelectionService::new());
        let anime_search_service = Arc::new(AnimeSearchService::new(
            provider_repository,
            cache_repository,
            (*data_quality_service).clone(),
        ));

        Self {
            anime_search_service,
            data_quality_service,
            provider_selection_service,
        }
    }

    /// Search anime across providers with smart data merging
    pub async fn search_anime(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>> {
        let criteria = SearchCriteria::new(query.to_string()).with_limit(limit);
        let available_providers = self.provider_selection_service.get_available_providers();

        let anime_data_results = self
            .anime_search_service
            .search(&criteria, &available_providers)
            .await?;

        // Convert AnimeData to AnimeDetailed
        let results = anime_data_results
            .into_iter()
            .map(|data| data.anime)
            .collect();
        Ok(results)
    }

    /// Get anime by ID from specific provider
    pub async fn get_anime_by_id(
        &self,
        id: &str,
        provider: AnimeProvider,
    ) -> AppResult<Option<AnimeDetailed>> {
        let available_providers = self.provider_selection_service.get_available_providers();

        match self
            .anime_search_service
            .get_details(id, Some(provider), &available_providers)
            .await?
        {
            Some(data) => Ok(Some(data.anime)),
            None => Ok(None),
        }
    }

    /// Check if a provider is healthy
    pub fn is_provider_healthy(&self, provider: &AnimeProvider) -> bool {
        self.provider_selection_service
            .get_health(provider)
            .map(|health| !health.should_avoid())
            .unwrap_or(false)
    }
}
