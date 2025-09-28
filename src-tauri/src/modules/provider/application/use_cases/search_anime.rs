use std::sync::Arc;
use std::time::Instant;

use crate::{
    modules::provider::{
        domain::{AnimeData, AnimeSearchService, ProviderSelectionService, SearchCriteria},
        AnimeProvider,
    },
    shared::errors::{AppError, AppResult},
};

use super::super::dto::{SearchAnimeRequest, SearchAnimeResponse};

/// Use case for searching anime across providers
pub struct SearchAnimeUseCase {
    search_service: Arc<AnimeSearchService>,
    provider_service: Arc<ProviderSelectionService>,
}

impl SearchAnimeUseCase {
    pub fn new(
        search_service: Arc<AnimeSearchService>,
        provider_service: Arc<ProviderSelectionService>,
    ) -> Self {
        Self {
            search_service,
            provider_service,
        }
    }

    /// Execute anime search with intelligent provider selection
    pub async fn execute(&self, request: SearchAnimeRequest) -> AppResult<SearchAnimeResponse> {
        let start_time = Instant::now();

        // Validate request
        if request.query.trim().is_empty() {
            return Err(AppError::InvalidInput(
                "Search query cannot be empty".to_string(),
            ));
        }

        // Create search criteria
        let criteria = SearchCriteria::new(request.query.clone())
            .with_limit(request.limit.unwrap_or(20))
            .with_quality_threshold(request.quality_threshold.unwrap_or(0.7));

        criteria.validate()?;

        // Get available providers
        let available_providers = self.provider_service.get_available_providers();
        if available_providers.is_empty() {
            return Err(AppError::ServiceUnavailable(
                "No providers available".to_string(),
            ));
        }

        log::info!(
            "Searching for '{}' using {} providers",
            criteria.query,
            available_providers.len()
        );

        // Execute search
        let search_results = self
            .search_service
            .search(&criteria, &available_providers)
            .await?;

        let search_duration = start_time.elapsed();

        // Store length before moving search_results
        let total_found = search_results.len();

        // Enhance results with multiple providers if requested and results are few
        let enhanced_results = if request.enhance_with_multiple_providers.unwrap_or(false)
            && total_found <= 3
            && available_providers.len() > 1
        {
            self.enhance_results_with_multiple_providers(search_results, &available_providers)
                .await?
        } else {
            search_results
        };

        // Create response
        let response = SearchAnimeResponse {
            results: enhanced_results,
            total_found,
            search_duration_ms: search_duration.as_millis() as u64,
            providers_used: available_providers,
            quality_threshold: criteria.quality_threshold,
        };

        log::info!(
            "Search completed in {}ms: {} results found",
            response.search_duration_ms,
            response.total_found
        );

        Ok(response)
    }

    /// Enhance search results by getting data from multiple providers
    async fn enhance_results_with_multiple_providers(
        &self,
        results: Vec<AnimeData>,
        available_providers: &[AnimeProvider],
    ) -> AppResult<Vec<AnimeData>> {
        let mut enhanced_results = Vec::new();

        for anime_data in results {
            // Find additional providers to enhance this result
            let additional_providers: Vec<AnimeProvider> = available_providers
                .iter()
                .filter(|&p| p != &anime_data.source.primary_provider)
                .cloned()
                .collect();

            if additional_providers.is_empty() {
                enhanced_results.push(anime_data);
                continue;
            }

            // Try to enhance with additional providers
            match self
                .search_service
                .enhance_with_multiple_providers(anime_data.clone(), &additional_providers)
                .await
            {
                Ok(enhanced) => enhanced_results.push(enhanced),
                Err(e) => {
                    log::warn!(
                        "Failed to enhance anime '{}': {}",
                        anime_data.anime.title.main,
                        e
                    );
                    enhanced_results.push(anime_data); // Use original data
                }
            }
        }

        Ok(enhanced_results)
    }
}
