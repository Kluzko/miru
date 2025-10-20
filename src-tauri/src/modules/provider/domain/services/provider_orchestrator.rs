use std::sync::Arc;
use std::time::Instant;

use crate::modules::provider::domain::{
    entities::AnimeData, repositories::AnimeProviderRepository, services::ProviderSelectionService,
    value_objects::SearchCriteria,
};
use crate::shared::{domain::value_objects::AnimeProvider, errors::AppResult};

/// Orchestrates provider queries using the Template Method pattern
///
/// This service is responsible for:
/// - Selecting which providers to query (using ProviderSelectionService)
/// - Executing queries across selected providers
/// - Handling provider failures gracefully
/// - Tracking timing and provider metadata
///
/// # Design Pattern: Template Method
/// The workflow is fixed (select providers → query providers), but the specifics
/// of provider selection can vary (preferred providers vs. auto-selection).
pub struct ProviderOrchestrator {
    provider_repo: Arc<dyn AnimeProviderRepository>,
    provider_selector: Arc<ProviderSelectionService>,
}

impl ProviderOrchestrator {
    pub fn new(
        provider_repo: Arc<dyn AnimeProviderRepository>,
        provider_selector: Arc<ProviderSelectionService>,
    ) -> Self {
        Self {
            provider_repo,
            provider_selector,
        }
    }

    /// Execute search across selected providers (Template Method)
    ///
    /// This is the main workflow:
    /// 1. Select providers based on criteria
    /// 2. Query each provider
    /// 3. Collect results (with graceful error handling)
    ///
    /// Returns grouped results: Vec<Vec<AnimeData>> where each inner Vec
    /// represents results from one provider.
    pub async fn execute_search(
        &self,
        criteria: &SearchCriteria,
    ) -> AppResult<Vec<Vec<AnimeData>>> {
        let start_time = Instant::now();

        // Step 1: Select providers (Strategy Pattern via ProviderSelectionService)
        let providers = self.select_providers(criteria);
        log::info!(
            "ORCHESTRATOR: Selected {} providers to query",
            providers.len()
        );

        // Step 2: Query providers
        let results = self.query_providers(providers, criteria, start_time).await;
        log::info!(
            "ORCHESTRATOR: Collected {} provider result groups",
            results.len()
        );

        Ok(results)
    }

    /// Execute details query across providers with fallback
    ///
    /// Tries providers in order until one succeeds, or all fail.
    pub async fn execute_details_query(
        &self,
        id: &str,
        preferred_provider: Option<AnimeProvider>,
        available_providers: &[AnimeProvider],
    ) -> AppResult<Option<AnimeData>> {
        let providers_to_try = if let Some(provider) = preferred_provider {
            vec![provider]
        } else {
            available_providers.to_vec()
        };

        for provider in providers_to_try {
            if !available_providers.contains(&provider) {
                continue;
            }

            // Check if provider is available
            if !self.provider_repo.is_provider_available(&provider).await {
                log::debug!("Provider {:?} is not available, skipping", provider);
                continue;
            }

            // Try to fetch from provider
            match self.provider_repo.get_anime_by_id(id, provider).await {
                Ok(Some(anime_data)) => {
                    log::debug!("Provider {:?} returned details for '{}'", provider, id);
                    return Ok(Some(anime_data));
                }
                Ok(None) => {
                    log::debug!("Provider {:?} found no details for '{}'", provider, id);
                    continue;
                }
                Err(e) => {
                    log::warn!("Provider {:?} failed for details '{}': {}", provider, id, e);
                    continue;
                }
            }
        }

        Ok(None)
    }

    // Private helper methods

    /// Select providers based on criteria
    ///
    /// Uses preferred providers from criteria if specified,
    /// otherwise delegates to ProviderSelectionService for smart selection.
    fn select_providers(&self, criteria: &SearchCriteria) -> Vec<AnimeProvider> {
        if !criteria.preferred_providers.is_empty() {
            log::debug!(
                "Using {} preferred providers from criteria",
                criteria.preferred_providers.len()
            );
            criteria.preferred_providers.clone()
        } else {
            log::debug!("No preferred providers, using auto-selection");
            self.provider_selector.get_ordered_providers()
        }
    }

    /// Query all selected providers
    ///
    /// Executes queries sequentially with graceful error handling.
    /// Failed providers are logged but don't stop the overall process.
    async fn query_providers(
        &self,
        providers: Vec<AnimeProvider>,
        criteria: &SearchCriteria,
        start_time: Instant,
    ) -> Vec<Vec<AnimeData>> {
        let mut all_results = Vec::new();
        let mut providers_tried = Vec::new();

        for provider in providers {
            // Check availability before querying
            if !self.provider_repo.is_provider_available(&provider).await {
                log::debug!("Provider {:?} is not available, skipping", provider);
                continue;
            }

            providers_tried.push(provider);

            // Query provider (caching happens transparently in decorator)
            match self
                .provider_repo
                .search_anime(&criteria.query, criteria.limit, provider)
                .await
            {
                Ok(mut results) => {
                    log::debug!(
                        "Provider {:?} returned {} results for '{}'",
                        provider,
                        results.len(),
                        criteria.query
                    );

                    // Enhance results with timing and provider metadata
                    let fetch_time_ms = start_time.elapsed().as_millis() as u64;
                    for result in &mut results {
                        result.source.fetch_time_ms = fetch_time_ms;
                        result.source.providers_used = providers_tried.clone();
                    }

                    all_results.push(results);
                }
                Err(e) => {
                    log::warn!(
                        "Provider {:?} failed for search '{}': {}",
                        provider,
                        criteria.query,
                        e
                    );
                    // Continue with other providers - graceful degradation
                    continue;
                }
            }
        }

        all_results
    }
}
