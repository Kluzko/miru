use crate::log_warn;
use crate::modules::anime::AnimeRepository;
use crate::shared::errors::{AppError, AppResult};
use crate::shared::utils::logger::{LogContext, TimedOperation};

use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::{ExistingAnime, ImportError, ValidatedAnime};
use crate::modules::provider::ProviderManager;

/// Handles validation logic using existing patterns
#[derive(Clone)]
pub struct ValidationService {
    anime_repo: Arc<dyn AnimeRepository>,
    provider_manager: Arc<Mutex<ProviderManager>>,
}

impl ValidationService {
    pub fn new(
        anime_repo: Arc<dyn AnimeRepository>,
        provider_manager: Arc<Mutex<ProviderManager>>,
    ) -> Self {
        Self {
            anime_repo,
            provider_manager,
        }
    }

    /// Helper method to get primary external ID and provider from anime (reused from existing)
    pub fn get_primary_external_info(
        anime: &crate::modules::anime::AnimeDetailed,
    ) -> (String, crate::modules::provider::AnimeProvider) {
        let provider = anime.provider_metadata.primary_provider.clone();
        let external_id = anime
            .provider_metadata
            .get_external_id(&provider)
            .cloned()
            .unwrap_or_else(|| "0".to_string());
        (external_id, provider)
    }

    /// Helper method to check if external ID is valid (reused from existing)
    pub fn is_valid_external_id(external_id: &str) -> bool {
        crate::shared::utils::Validator::is_valid_external_id(external_id)
    }

    /// Search for anime using provider manager with fallback (reused from existing)
    pub async fn search_anime_multi_provider(
        &self,
        query: &str,
    ) -> AppResult<Vec<crate::modules::anime::AnimeDetailed>> {
        // TODO: In production, refactor ProviderManager to avoid lock-across-await
        // For now, keeping the existing pattern but noting the issue
        let mut provider_manager = self.provider_manager.lock().await;

        match provider_manager.search_anime(query, 1).await {
            Ok(results) if !results.is_empty() => {
                LogContext::search_operation(query, Some("provider_manager"), Some(results.len()));
                Ok(results)
            }
            Ok(_) => {
                LogContext::search_operation(query, Some("provider_manager"), Some(0));
                Ok(vec![])
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    &format!("Provider search failed for '{}'", query),
                );
                Err(e)
            }
        }
    }

    /// Validate a single anime title using existing DB-first logic
    pub async fn validate_single_title(&self, title: &str) -> ValidationSingleResult {
        let item_timer = TimedOperation::new("validate_single_title");

        // STEP 1: Check database first using title variations (reused from existing)
        match self.anime_repo.find_by_title_variations(title).await {
            Ok(Some(existing_anime)) => {
                item_timer.finish();
                return ValidationSingleResult::AlreadyExists(ExistingAnime {
                    input_title: title.to_string(),
                    matched_title: existing_anime.title.main.clone(),
                    matched_field: "title_match".to_string(),
                    anime: existing_anime,
                });
            }
            Ok(None) => {
                // Continue to provider lookup
            }
            Err(e) => {
                log_warn!("Database lookup failed for '{}': {}", title, e);
                // Continue to provider lookup as fallback
            }
        }

        // STEP 2: Search providers only if not found in DB (reused from existing)
        match self.search_anime_multi_provider(title).await {
            Ok(anime_list) if !anime_list.is_empty() => {
                let anime = anime_list.into_iter().next().unwrap();
                let (external_id, provider) = Self::get_primary_external_info(&anime);

                if Self::is_valid_external_id(&external_id) {
                    // STEP 3: Double-check by external_id to avoid duplicates (reused from existing)
                    match self
                        .anime_repo
                        .find_by_external_id(&provider, &external_id)
                        .await
                    {
                        Ok(Some(existing)) => {
                            item_timer.finish();
                            ValidationSingleResult::AlreadyExists(ExistingAnime {
                                input_title: title.to_string(),
                                matched_title: existing.title.main.clone(),
                                matched_field: format!("{:?}_id", provider),
                                anime: existing,
                            })
                        }
                        Ok(None) => {
                            item_timer.finish();
                            ValidationSingleResult::Found(ValidatedAnime {
                                input_title: title.to_string(),
                                anime_data: anime,
                            })
                        }
                        Err(e) => {
                            let context_error = AppError::DatabaseError(format!(
                                "External ID lookup failed for '{}' with provider {:?}: {}",
                                title, provider, e
                            ));
                            LogContext::error_with_context(
                                &context_error,
                                "External ID validation failed",
                            );
                            item_timer.finish();
                            ValidationSingleResult::Failed(ImportError {
                                title: title.to_string(),
                                reason: format!("Database error during external ID check: {}", e),
                            })
                        }
                    }
                } else {
                    log_warn!("Invalid external ID for '{}': {}", title, external_id);
                    item_timer.finish();
                    ValidationSingleResult::Found(ValidatedAnime {
                        input_title: title.to_string(),
                        anime_data: anime,
                    })
                }
            }
            Ok(_) => {
                item_timer.finish();
                ValidationSingleResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: "No results found on any provider".to_string(),
                })
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    &format!("Provider search failed for '{}'", title),
                );
                item_timer.finish();
                ValidationSingleResult::Failed(ImportError {
                    title: title.to_string(),
                    reason: format!("Provider search failed: {}", e),
                })
            }
        }
    }
}

/// Result type for single validation operations
#[derive(Debug)]
pub enum ValidationSingleResult {
    Found(ValidatedAnime),
    AlreadyExists(ExistingAnime),
    Failed(ImportError),
}
