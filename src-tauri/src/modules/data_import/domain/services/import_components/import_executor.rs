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

        // Note: We now use smart duplicate detection instead of just external ID checking

        // STEP 1: Simple duplicate check using provider metadata
        // Check if anime already exists by external ID
        // Check if anime already exists by external ID for any provider
        for (provider, external_id) in &validated.anime_data.provider_metadata.external_ids {
            if let Ok(existing_anime_list) = self.anime_repo.get_all(0, 10000).await {
                for existing in existing_anime_list {
                    if let Some(existing_id) = existing.provider_metadata.get_external_id(provider)
                    {
                        if existing_id == external_id {
                            log_info!(
                                "Skipping '{}' - already exists with {} ID: {}",
                                anime_title,
                                provider,
                                external_id
                            );
                            item_timer.finish();
                            return Err(ImportError {
                                title: validated.input_title.clone(),
                                reason: format!(
                                    "Already exists with {} ID: {}",
                                    provider, external_id
                                ),
                            });
                        }
                    }
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
