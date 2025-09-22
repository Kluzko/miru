use crate::modules::anime::AnimeRepository;
use crate::shared::errors::AppResult;

use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::modules::provider::ProviderService;

use super::super::domain::services::import_components::{
    BatchQualityInsights, DataEnhancementService, EnhancedValidationResult, ImportCoordinator,
    ImportResult, ValidatedAnime, ValidationResult,
};

/// Import service - Clean interface that delegates to focused components
///
/// This service provides a unified interface for all import operations while
/// delegating the actual work to specialized components for better maintainability.
#[derive(Clone)]
pub struct ImportService {
    anime_repo: Arc<dyn AnimeRepository>,
    provider_service: Arc<ProviderService>,
}

impl ImportService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        provider_service: Arc<ProviderService>,
    ) -> Self {
        Self {
            anime_repo,
            provider_service,
        }
    }

    /// Enhanced import anime batch with comprehensive provider data aggregation
    pub async fn import_anime_batch_enhanced(
        &self,
        titles: Vec<String>,
        app_handle: Option<tauri::AppHandle>,
        _cancellation_token: Option<CancellationToken>,
    ) -> AppResult<(ImportResult, BatchQualityInsights)> {
        let coordinator = ImportCoordinator::new(
            self.anime_repo.clone(),
            self.provider_service.clone(),
            app_handle,
        );

        // Step 1: Enhanced validation with comprehensive provider data
        let enhanced_validation_result = coordinator.validate_anime_titles_enhanced(titles).await?;

        // Step 2: Data enhancement for quality improvement
        let enhancement_service = DataEnhancementService::new(self.provider_service.clone());
        let (_enhancement_results, quality_insights) = enhancement_service
            .enhance_batch(enhanced_validation_result.found.clone())
            .await?;

        // Step 3: Import enhanced validated anime
        let import_result = coordinator
            .import_enhanced_validated_anime(enhanced_validation_result.found)
            .await?;

        Ok((import_result, quality_insights))
    }

    /// Import anime batch with progress reporting and dynamic concurrency optimization
    pub async fn import_anime_batch(
        &self,
        titles: Vec<String>,
        app_handle: Option<tauri::AppHandle>,
        _cancellation_token: Option<CancellationToken>,
    ) -> AppResult<ImportResult> {
        let coordinator = ImportCoordinator::new(
            self.anime_repo.clone(),
            self.provider_service.clone(),
            app_handle,
        );

        coordinator.import_anime_batch(titles).await
    }

    /// Enhanced validate anime titles with comprehensive provider data aggregation
    pub async fn validate_anime_titles_enhanced(
        &self,
        titles: Vec<String>,
        app_handle: Option<&tauri::AppHandle>,
    ) -> AppResult<EnhancedValidationResult> {
        let coordinator = ImportCoordinator::new(
            self.anime_repo.clone(),
            self.provider_service.clone(),
            app_handle.cloned(),
        );

        coordinator.validate_anime_titles_enhanced(titles).await
    }

    /// Validate anime titles with optimized DB-first lookup and batched progress events
    pub async fn validate_anime_titles(
        &self,
        titles: Vec<String>,
        app_handle: Option<&tauri::AppHandle>,
    ) -> AppResult<ValidationResult> {
        let coordinator = ImportCoordinator::new(
            self.anime_repo.clone(),
            self.provider_service.clone(),
            app_handle.cloned(),
        );

        coordinator.validate_anime_titles(titles).await
    }

    /// Import validated anime to database with dynamic concurrency optimization
    pub async fn import_validated_anime(
        &self,
        validated_anime: Vec<ValidatedAnime>,
        app_handle: Option<tauri::AppHandle>,
    ) -> AppResult<ImportResult> {
        let coordinator = ImportCoordinator::new(
            self.anime_repo.clone(),
            self.provider_service.clone(),
            app_handle,
        );

        coordinator.import_validated_anime(validated_anime).await
    }
}
