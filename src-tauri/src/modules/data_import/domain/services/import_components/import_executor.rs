use crate::log_info;
use crate::modules::anime::{AnimeRepository, AnimeService};
use crate::shared::errors::AppError;
use crate::shared::utils::logger::{LogContext, TimedOperation};

use std::sync::Arc;

use super::types::{ImportError, ImportedAnime, ValidatedAnime};
use super::validation_service::ValidationService;

/// Handles import execution using proper service layer for clean architecture
#[derive(Clone)]
pub struct ImportExecutor {
    anime_repo: Arc<dyn AnimeRepository>,
    anime_service: Arc<AnimeService>,
}

impl ImportExecutor {
    pub fn new(anime_repo: Arc<dyn AnimeRepository>, anime_service: Arc<AnimeService>) -> Self {
        Self {
            anime_repo,
            anime_service,
        }
    }

    /// Execute import for a single validated anime using existing logic
    pub async fn import_single_validated(
        &self,
        validated: &ValidatedAnime,
    ) -> Result<ImportedAnime, ImportError> {
        let item_timer = TimedOperation::new("import_single_validated_anime");
        let anime_title = &validated.anime_data.title.main;

        // Get external ID and provider for duplicate checking (reused from existing)
        let (external_id, provider) =
            ValidationService::get_primary_external_info(&validated.anime_data);

        // STEP 1: Double-check for duplicates using external ID (reused from existing)
        if ValidationService::is_valid_external_id(&external_id) {
            match self
                .anime_repo
                .find_by_external_id(&provider, &external_id)
                .await
            {
                Ok(Some(_existing_anime)) => {
                    log_info!(
                        "Skipping '{}' - already exists in database with {} ID: {}",
                        anime_title,
                        match provider {
                            crate::modules::provider::AnimeProvider::Jikan => "MAL",
                            crate::modules::provider::AnimeProvider::AniList => "AniList",
                            _ => "Unknown",
                        },
                        external_id
                    );
                    item_timer.finish();
                    return Err(ImportError {
                        title: validated.input_title.clone(),
                        reason: "Already exists in database".to_string(),
                    });
                }
                Ok(None) => {
                    log_info!(
                        "External ID check passed for '{}', proceeding with save",
                        anime_title
                    );
                }
                Err(e) => {
                    let context_error = AppError::DatabaseError(format!(
                        "Pre-import external ID check failed for '{}' with provider {:?}: {}",
                        anime_title, provider, e
                    ));
                    LogContext::error_with_context(
                        &context_error,
                        "Pre-import duplicate check failed",
                    );
                    // Continue with import as fallback
                }
            }
        }

        // STEP 2: Save to database using service layer for proper score calculation
        match self.anime_service.create_anime(&validated.anime_data).await {
            Ok(saved_anime) => {
                let (saved_external_id, saved_provider) =
                    ValidationService::get_primary_external_info(&saved_anime);

                log_info!(
                    "Successfully imported '{}' from {} with ID: {}",
                    saved_anime.title.main,
                    match saved_provider {
                        crate::modules::provider::AnimeProvider::Jikan => "MAL",
                        crate::modules::provider::AnimeProvider::AniList => "AniList",
                        _ => "Unknown",
                    },
                    saved_external_id
                );

                item_timer.finish();
                Ok(ImportedAnime {
                    title: saved_anime.title.main.clone(),
                    primary_external_id: saved_external_id,
                    provider: saved_provider,
                    id: saved_anime.id,
                })
            }
            Err(AppError::Duplicate(msg)) => {
                log_info!(
                    "Duplicate detected during save for '{}': {}",
                    anime_title,
                    msg
                );
                item_timer.finish();
                Err(ImportError {
                    title: validated.input_title.clone(),
                    reason: "Duplicate detected during save".to_string(),
                })
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    &format!("Failed to save validated anime '{}'", anime_title),
                );
                item_timer.finish();
                Err(ImportError {
                    title: validated.input_title.clone(),
                    reason: format!("Database error: {}", e),
                })
            }
        }
    }
}
