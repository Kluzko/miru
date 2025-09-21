use crate::modules::anime::AnimeRepository;
use crate::shared::errors::AppResult;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::modules::provider::ProviderManager;

use super::super::domain::services::import_components::{
    ImportCoordinator, ImportResult, ValidatedAnime, ValidationResult,
};

/// Import service - Clean interface that delegates to focused components
///
/// This service provides a unified interface for all import operations while
/// delegating the actual work to specialized components for better maintainability.
#[derive(Clone)]
pub struct ImportService {
    anime_repo: Arc<dyn AnimeRepository>,
    provider_manager: Arc<Mutex<ProviderManager>>,
}

impl ImportService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        provider_manager: Arc<Mutex<ProviderManager>>,
    ) -> Self {
        Self {
            anime_repo,
            provider_manager,
        }
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
            self.provider_manager.clone(),
            app_handle,
        );

        coordinator.import_anime_batch(titles).await
    }

    /// Validate anime titles with optimized DB-first lookup and batched progress events
    pub async fn validate_anime_titles(
        &self,
        titles: Vec<String>,
        app_handle: Option<&tauri::AppHandle>,
    ) -> AppResult<ValidationResult> {
        let coordinator = ImportCoordinator::new(
            self.anime_repo.clone(),
            self.provider_manager.clone(),
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
            self.provider_manager.clone(),
            app_handle,
        );

        coordinator.import_validated_anime(validated_anime).await
    }
}
