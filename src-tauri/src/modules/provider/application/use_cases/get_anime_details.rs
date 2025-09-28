use std::sync::Arc;
use std::time::Instant;

use crate::{
    modules::provider::{
        domain::services::OperationType,
        domain::{AnimeSearchService, GetDetailsCriteria, ProviderSelectionService},
        AnimeProvider,
    },
    shared::errors::{AppError, AppResult},
};

use super::super::dto::{GetAnimeDetailsRequest, GetAnimeDetailsResponse};

/// Use case for getting detailed anime information
pub struct GetAnimeDetailsUseCase {
    search_service: Arc<AnimeSearchService>,
    provider_service: Arc<ProviderSelectionService>,
}

impl GetAnimeDetailsUseCase {
    pub fn new(
        search_service: Arc<AnimeSearchService>,
        provider_service: Arc<ProviderSelectionService>,
    ) -> Self {
        Self {
            search_service,
            provider_service,
        }
    }

    /// Execute get anime details operation
    pub async fn execute(
        &self,
        request: GetAnimeDetailsRequest,
    ) -> AppResult<GetAnimeDetailsResponse> {
        let start_time = Instant::now();

        // Create criteria
        let criteria = GetDetailsCriteria::new(request.id.clone()).with_provider(
            request.preferred_provider.unwrap_or_else(|| {
                // Auto-select best provider for details
                self.provider_service
                    .select_best_provider(OperationType::GetDetails)
                    .unwrap_or(AnimeProvider::AniList)
            }),
        );

        criteria.validate()?;

        // Get available providers
        let available_providers = self.provider_service.get_available_providers();
        if available_providers.is_empty() {
            return Err(AppError::ServiceUnavailable(
                "No providers available".to_string(),
            ));
        }

        log::info!(
            "Getting details for ID '{}' using providers: {:?}",
            criteria.id,
            available_providers
        );

        // Execute get details
        let anime_result = self
            .search_service
            .get_details(&criteria.id, criteria.provider, &available_providers)
            .await?;

        let fetch_duration = start_time.elapsed();

        // Enhance with multiple providers if requested and anime was found
        let enhanced_anime = if let Some(anime_data) = anime_result {
            if request.enhance_with_multiple_providers.unwrap_or(false)
                && available_providers.len() > 1
            {
                let additional_providers: Vec<AnimeProvider> = available_providers
                    .iter()
                    .filter(|&p| Some(*p) != criteria.provider)
                    .cloned()
                    .collect();

                match self
                    .search_service
                    .enhance_with_multiple_providers(anime_data.clone(), &additional_providers)
                    .await
                {
                    Ok(enhanced) => Some(enhanced),
                    Err(e) => {
                        log::warn!(
                            "Failed to enhance anime details for '{}': {}",
                            criteria.id,
                            e
                        );
                        Some(anime_data) // Use original data
                    }
                }
            } else {
                Some(anime_data)
            }
        } else {
            None
        };

        // Create response
        let found = enhanced_anime.is_some();
        let response = GetAnimeDetailsResponse {
            anime: enhanced_anime,
            fetch_duration_ms: fetch_duration.as_millis() as u64,
            provider_used: criteria.provider,
            found,
        };

        if response.found {
            log::info!(
                "Details fetched in {}ms for ID '{}'",
                response.fetch_duration_ms,
                criteria.id
            );
        } else {
            log::info!(
                "No details found for ID '{}' after {}ms",
                criteria.id,
                response.fetch_duration_ms
            );
        }

        Ok(response)
    }

    /// Get anime details with automatic fallback across all providers
    pub async fn execute_with_fallback(
        &self,
        request: GetAnimeDetailsRequest,
    ) -> AppResult<GetAnimeDetailsResponse> {
        let start_time = Instant::now();

        let available_providers = self.provider_service.get_available_providers();
        if available_providers.is_empty() {
            return Err(AppError::ServiceUnavailable(
                "No providers available".to_string(),
            ));
        }

        // Try each provider until we find the anime
        for provider in &available_providers {
            log::debug!(
                "Trying provider {:?} for anime ID '{}'",
                provider,
                request.id
            );

            match self
                .search_service
                .get_details(&request.id, Some(*provider), &available_providers)
                .await
            {
                Ok(Some(anime_data)) => {
                    log::info!("Found anime details with provider {:?}", provider);

                    let response = GetAnimeDetailsResponse {
                        anime: Some(anime_data),
                        fetch_duration_ms: start_time.elapsed().as_millis() as u64,
                        provider_used: Some(*provider),
                        found: true,
                    };

                    return Ok(response);
                }
                Ok(None) => {
                    log::debug!(
                        "Provider {:?} returned no results for '{}'",
                        provider,
                        request.id
                    );
                    continue;
                }
                Err(e) => {
                    log::warn!("Provider {:?} failed for '{}': {}", provider, request.id, e);
                    continue;
                }
            }
        }

        // No provider found the anime
        let response = GetAnimeDetailsResponse {
            anime: None,
            fetch_duration_ms: start_time.elapsed().as_millis() as u64,
            provider_used: None,
            found: false,
        };

        Ok(response)
    }
}
