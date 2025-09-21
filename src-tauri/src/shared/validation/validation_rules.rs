use super::validation_chain::{ValidationContext, ValidationResult, ValidationRule};
use crate::modules::anime::AnimeRepository;
use crate::modules::provider::AnimeProvider;
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use chrono::Datelike;
use std::sync::Arc;

/// Validates basic anime title requirements
pub struct TitleValidationRule;

#[async_trait]
impl ValidationRule for TitleValidationRule {
    async fn validate(&self, context: &ValidationContext) -> AppResult<ValidationResult> {
        let anime = &context.anime;
        let mut result = ValidationResult::valid();

        // Check if title exists
        if anime.title.main.trim().is_empty() {
            return Ok(ValidationResult::invalid(
                "Anime title cannot be empty".to_string(),
            ));
        }

        // Check title length
        if anime.title.main.len() > 255 {
            return Ok(ValidationResult::invalid(
                "Anime title too long (max 255 characters)".to_string(),
            ));
        }

        // Warning for very short titles
        if anime.title.main.len() < 3 {
            result = result
                .with_warning("Title is very short, might not be specific enough".to_string());
        }

        Ok(result)
    }

    fn rule_name(&self) -> &'static str {
        "TitleValidation"
    }
}

/// Validates anime scoring and rating data
pub struct ScoreValidationRule;

#[async_trait]
impl ValidationRule for ScoreValidationRule {
    async fn validate(&self, context: &ValidationContext) -> AppResult<ValidationResult> {
        let anime = &context.anime;
        let mut result = ValidationResult::valid();

        // Validate score if present
        if let Some(score) = anime.score {
            if !(0.0..=10.0).contains(&score) {
                return Ok(ValidationResult::invalid(
                    "Score must be between 0 and 10".to_string(),
                ));
            }

            // Warning for extreme scores
            if score < 1.0 || score > 9.5 {
                result =
                    result.with_warning("Score is in extreme range, please verify".to_string());
            }
        }

        // Validate favorites count
        if let Some(favorites) = anime.favorites {
            if favorites < 0 {
                return Ok(ValidationResult::invalid(
                    "Favorites count cannot be negative".to_string(),
                ));
            }
        }

        Ok(result)
    }

    fn rule_name(&self) -> &'static str {
        "ScoreValidation"
    }
}

/// Validates external ID and provider-specific requirements
pub struct ExternalIdValidationRule;

#[async_trait]
impl ValidationRule for ExternalIdValidationRule {
    async fn validate(&self, context: &ValidationContext) -> AppResult<ValidationResult> {
        let anime = &context.anime;
        let provider = &context.provider;

        // Get external ID for this provider
        let default_id = String::new();
        let external_id = anime
            .provider_metadata
            .get_external_id(provider)
            .unwrap_or(&default_id);

        if external_id.is_empty() || external_id == "0" {
            return Ok(ValidationResult::invalid(
                "External ID cannot be empty or '0'".to_string(),
            ));
        }

        // Provider-specific validation
        match provider {
            AnimeProvider::Jikan => match external_id.parse::<i32>() {
                Ok(id) if id > 0 => {}
                _ => {
                    return Ok(ValidationResult::invalid(
                        "Jikan (MAL) ID must be a positive integer".to_string(),
                    ))
                }
            },
            AnimeProvider::AniList => match external_id.parse::<i32>() {
                Ok(id) if id > 0 => {}
                _ => {
                    return Ok(ValidationResult::invalid(
                        "AniList ID must be a positive integer".to_string(),
                    ))
                }
            },
            _ => {
                // For other providers, basic validation
                if external_id.trim().is_empty() {
                    return Ok(ValidationResult::invalid(
                        "External ID cannot be empty".to_string(),
                    ));
                }
            }
        }

        Ok(ValidationResult::valid())
    }

    fn rule_name(&self) -> &'static str {
        "ExternalIdValidation"
    }

    fn should_skip(&self, context: &ValidationContext) -> bool {
        !context.provider_specific_checks
    }
}

/// Checks for duplicate anime in the database
pub struct DuplicateCheckRule {
    anime_repo: Arc<dyn AnimeRepository>,
}

impl DuplicateCheckRule {
    pub fn new(anime_repo: Arc<dyn AnimeRepository>) -> Self {
        Self { anime_repo }
    }
}

#[async_trait]
impl ValidationRule for DuplicateCheckRule {
    async fn validate(&self, context: &ValidationContext) -> AppResult<ValidationResult> {
        let anime = &context.anime;
        let provider = &context.provider;

        // Get external ID for this provider
        let default_id = String::new();
        let external_id = anime
            .provider_metadata
            .get_external_id(provider)
            .unwrap_or(&default_id);

        if !external_id.is_empty() && external_id != "0" {
            // Check by external ID first
            match self
                .anime_repo
                .find_by_external_id(provider, external_id)
                .await?
            {
                Some(existing) => {
                    return Ok(ValidationResult::invalid(format!(
                        "Anime already exists in database with {} ID: {} (Title: '{}')",
                        match provider {
                            AnimeProvider::Jikan => "MAL",
                            AnimeProvider::AniList => "AniList",
                            _ => "External",
                        },
                        external_id,
                        existing.title.main
                    )));
                }
                None => {}
            }
        }

        // Check by title variations as fallback
        match self
            .anime_repo
            .find_by_title_variations(&anime.title.main)
            .await?
        {
            Some(existing) => {
                let mut result = ValidationResult::valid();
                result = result.with_warning(format!(
                    "Similar anime found in database: '{}' - please verify this is not a duplicate",
                    existing.title.main
                ));
                Ok(result)
            }
            None => Ok(ValidationResult::valid()),
        }
    }

    fn rule_name(&self) -> &'static str {
        "DuplicateCheck"
    }
}

/// Validates episode count and series information
pub struct EpisodeValidationRule;

#[async_trait]
impl ValidationRule for EpisodeValidationRule {
    async fn validate(&self, context: &ValidationContext) -> AppResult<ValidationResult> {
        let anime = &context.anime;
        let mut result = ValidationResult::valid();

        if let Some(episodes) = anime.episodes {
            if episodes < 0 {
                return Ok(ValidationResult::invalid(
                    "Episode count cannot be negative".to_string(),
                ));
            }

            if episodes == 0 {
                result = result
                    .with_warning("Episode count is 0 - this might be incomplete data".to_string());
            }

            // Warning for very high episode counts
            if episodes > 1000 {
                result = result.with_warning(
                    "Very high episode count - please verify this is correct".to_string(),
                );
            }
        }

        Ok(result)
    }

    fn rule_name(&self) -> &'static str {
        "EpisodeValidation"
    }
}

/// Validates date information (aired dates)
pub struct DateValidationRule;

#[async_trait]
impl ValidationRule for DateValidationRule {
    async fn validate(&self, context: &ValidationContext) -> AppResult<ValidationResult> {
        let anime = &context.anime;
        let mut result = ValidationResult::valid();

        // Check if aired_to is before aired_from
        if let (Some(aired_from), Some(aired_to)) = (&anime.aired.from, &anime.aired.to) {
            if aired_to < aired_from {
                return Ok(ValidationResult::invalid(
                    "End date cannot be before start date".to_string(),
                ));
            }
        }

        // Warning for very old anime (might be incorrect data)
        if let Some(aired_from) = &anime.aired.from {
            let current_year = chrono::Utc::now().year();
            let aired_year = aired_from.year();

            if aired_year < 1900 {
                return Ok(ValidationResult::invalid(
                    "Aired date appears to be invalid (before 1900)".to_string(),
                ));
            }

            if aired_year > current_year + 2 {
                result = result.with_warning(
                    "Anime is scheduled for far future - please verify date".to_string(),
                );
            }
        }

        Ok(result)
    }

    fn rule_name(&self) -> &'static str {
        "DateValidation"
    }
}

/// Factory for creating validation chains
pub struct ValidationChainBuilder {
    anime_repo: Option<Arc<dyn AnimeRepository>>,
}

impl ValidationChainBuilder {
    pub fn new() -> Self {
        Self { anime_repo: None }
    }

    pub fn with_anime_repository(mut self, repo: Arc<dyn AnimeRepository>) -> Self {
        self.anime_repo = Some(repo);
        self
    }

    /// Build a standard validation chain for anime import
    pub fn build_import_chain(self) -> super::ValidationChain {
        let mut chain = super::ValidationChain::new()
            .add_rule(Arc::new(TitleValidationRule))
            .add_rule(Arc::new(ExternalIdValidationRule))
            .add_rule(Arc::new(ScoreValidationRule))
            .add_rule(Arc::new(EpisodeValidationRule))
            .add_rule(Arc::new(DateValidationRule));

        // Add duplicate check if repository is available
        if let Some(repo) = self.anime_repo {
            chain = chain.add_rule(Arc::new(DuplicateCheckRule::new(repo)));
        }

        chain
    }

    /// Build a strict validation chain (stops on first error)
    pub fn build_strict_chain(self) -> super::ValidationChain {
        self.build_import_chain().stop_on_first_error(true)
    }

    /// Build a lightweight validation chain (no database checks)
    pub fn build_lightweight_chain(self) -> super::ValidationChain {
        super::ValidationChain::new()
            .add_rule(Arc::new(TitleValidationRule))
            .add_rule(Arc::new(ExternalIdValidationRule))
            .add_rule(Arc::new(ScoreValidationRule))
    }
}

impl Default for ValidationChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}
