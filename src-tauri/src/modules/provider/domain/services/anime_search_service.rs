use std::sync::Arc;

use crate::{
    modules::{
        anime::domain::services::data_quality_service::DataQualityService,
        provider::{
            domain::{
                entities::AnimeData,
                repositories::AnimeProviderRepository,
                services::{
                    ProviderOrchestrator, ProviderSelectionService, SearchResultsProcessor,
                },
                value_objects::SearchCriteria,
            },
            AnimeProvider,
        },
    },
    shared::errors::AppResult,
};

/// Simplified anime search service that coordinates search operations
///
/// This service has been refactored from a 749-line "God Service" into a lean
/// coordinator that delegates to specialized services:
///
/// - **ProviderOrchestrator**: Handles provider selection and querying
/// - **SearchResultsProcessor**: Handles deduplication, merging, ranking, filtering
/// - **DataQualityService**: Handles data quality assessment and merging
///
/// # Design Pattern: Facade + Coordinator
/// This service provides a simple interface for search operations while
/// orchestrating multiple domain services behind the scenes.
///
/// # Architecture:
/// - **Before**: 749 lines, 7+ responsibilities (search, cache, merge, dedup, rank, filter, provider selection)
/// - **After**: ~100 lines, 1 responsibility (coordinate workflow)
pub struct AnimeSearchService {
    orchestrator: Arc<ProviderOrchestrator>,
    processor: Arc<SearchResultsProcessor>,
    provider_repo: Arc<dyn AnimeProviderRepository>,
    quality_service: Arc<DataQualityService>,
}

impl AnimeSearchService {
    /// Create a new AnimeSearchService
    ///
    /// This constructor builds the service graph:
    /// - Creates ProviderOrchestrator (for provider queries)
    /// - Creates SearchResultsProcessor (for result processing)
    pub fn new(
        provider_repo: Arc<dyn AnimeProviderRepository>,
        quality_service: DataQualityService,
    ) -> Self {
        // Build service dependencies
        let provider_selector = Arc::new(ProviderSelectionService::new());
        let orchestrator = Arc::new(ProviderOrchestrator::new(
            provider_repo.clone(),
            provider_selector,
        ));

        let quality_service_arc = Arc::new(quality_service);
        let processor = Arc::new(SearchResultsProcessor::new(quality_service_arc.clone()));

        Self {
            orchestrator,
            processor,
            provider_repo,
            quality_service: quality_service_arc,
        }
    }

    /// Search for anime using intelligent provider selection with multi-provider data merging
    ///
    /// This is now a clean coordination workflow:
    /// 1. Validate criteria
    /// 2. Orchestrate provider queries (delegate to ProviderOrchestrator)
    /// 3. Process results (delegate to SearchResultsProcessor)
    pub async fn search(
        &self,
        criteria: &SearchCriteria,
        _available_providers: &[AnimeProvider], // Not used anymore - orchestrator handles this
    ) -> AppResult<Vec<AnimeData>> {
        // Step 1: Validate search criteria
        criteria.validate()?;
        log::info!("SEARCH: Starting search for '{}'", criteria.query);

        // Step 2: Orchestrate provider queries
        let provider_results = self.orchestrator.execute_search(criteria).await?;

        // Early return if no results
        if provider_results.is_empty() {
            log::info!("SEARCH: No provider returned results");
            return Ok(Vec::new());
        }

        // Step 3: Process results through pipeline
        let final_results = self.processor.process(provider_results, criteria).await?;

        log::info!(
            "SEARCH: Completed search for '{}' with {} final results",
            criteria.query,
            final_results.len()
        );

        Ok(final_results)
    }

    /// Get anime details with fallback across providers
    ///
    /// Delegates to ProviderOrchestrator for provider querying with fallback.
    pub async fn get_details(
        &self,
        id: &str,
        preferred_provider: Option<AnimeProvider>,
        available_providers: &[AnimeProvider],
    ) -> AppResult<Option<AnimeData>> {
        log::info!("DETAILS: Fetching details for id '{}'", id);

        // Delegate to orchestrator
        let result = self
            .orchestrator
            .execute_details_query(id, preferred_provider, available_providers)
            .await?;

        if result.is_some() {
            log::info!("DETAILS: Successfully fetched details for '{}'", id);
        } else {
            log::info!("DETAILS: No provider found details for '{}'", id);
        }

        Ok(result)
    }

    /// Enhance anime data by fetching from additional providers and merging
    ///
    /// This method is used by use cases to optionally enhance a single anime's data
    /// by querying additional providers and merging the results.
    ///
    /// # Arguments
    /// * `base_anime` - The base anime data to enhance
    /// * `additional_providers` - Providers to query for enhancement (excluding base provider)
    pub async fn enhance_with_multiple_providers(
        &self,
        base_anime: AnimeData,
        additional_providers: &[AnimeProvider],
    ) -> AppResult<AnimeData> {
        let mut all_data = vec![base_anime.clone()];

        // Try to get the same anime from other providers for data enhancement
        let search_query = &base_anime.anime.title.main;

        for &provider in additional_providers {
            if provider == base_anime.source.primary_provider {
                continue; // Skip the provider we already have
            }

            if !self.provider_repo.is_provider_available(&provider).await {
                continue;
            }

            match self
                .provider_repo
                .search_anime(search_query, 1, provider)
                .await
            {
                Ok(results) if !results.is_empty() => {
                    // Take the first result as it should be most relevant
                    let mut additional_data = results.into_iter().next().unwrap();
                    additional_data.source.providers_used =
                        all_data[0].source.providers_used.clone();
                    additional_data.source.providers_used.push(provider);
                    all_data.push(additional_data);
                }
                _ => continue,
            }
        }

        if all_data.len() == 1 {
            // No additional data found
            return Ok(base_anime);
        }

        // Merge data from multiple providers using quality service
        self.quality_service.merge_anime_data(all_data)
    }
}
