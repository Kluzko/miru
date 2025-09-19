use regex::Regex;
use std::sync::OnceLock;

use crate::shared::errors::AppError;

/// Regex pattern for validating collection names
/// Allows alphanumeric characters, spaces, hyphens, and underscores
const COLLECTION_NAME_PATTERN: &str = r"^[a-zA-Z0-9\s\-_]+$";

/// Get compiled regex for collection name validation
fn get_collection_name_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(COLLECTION_NAME_PATTERN).expect("Collection name regex pattern is invalid")
    })
}

pub struct Validator;

impl Validator {
    pub fn validate_anime_title(title: &str) -> Result<(), AppError> {
        if title.is_empty() {
            return Err(AppError::ValidationError(
                "Title cannot be empty".to_string(),
            ));
        }
        if title.len() > 255 {
            return Err(AppError::ValidationError(
                "Title too long (max 255 characters)".to_string(),
            ));
        }
        Ok(())
    }

    pub fn validate_score(score: f32) -> Result<(), AppError> {
        if !(0.0..=10.0).contains(&score) {
            return Err(AppError::ValidationError(
                "Score must be between 0 and 10".to_string(),
            ));
        }
        Ok(())
    }

    pub fn validate_collection_name(name: &str) -> Result<(), AppError> {
        if name.is_empty() {
            return Err(AppError::ValidationError(
                "Collection name cannot be empty".to_string(),
            ));
        }
        if name.len() > 100 {
            return Err(AppError::ValidationError(
                "Collection name too long (max 100 characters)".to_string(),
            ));
        }

        // Check for valid characters (alphanumeric, spaces, and some special characters)
        if !get_collection_name_regex().is_match(name) {
            return Err(AppError::ValidationError(
                "Collection name contains invalid characters".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate external ID for any provider
    pub fn validate_external_id(
        external_id: &str,
        provider: &crate::domain::value_objects::AnimeProvider,
    ) -> Result<(), AppError> {
        if external_id.is_empty() {
            return Err(AppError::ValidationError(
                "External ID cannot be empty".to_string(),
            ));
        }

        if external_id == "0" {
            return Err(AppError::ValidationError(
                "External ID cannot be '0'".to_string(),
            ));
        }

        // Provider-specific validation
        match provider {
            crate::domain::value_objects::AnimeProvider::Jikan => {
                // MAL IDs should be positive integers
                match external_id.parse::<i32>() {
                    Ok(id) if id > 0 => {} // Valid positive integer
                    _ => {
                        return Err(AppError::ValidationError(
                            "Jikan (MAL) ID must be a positive integer".to_string(),
                        ))
                    }
                }
            }
            crate::domain::value_objects::AnimeProvider::AniList => {
                // AniList IDs should be positive integers
                match external_id.parse::<i32>() {
                    Ok(id) if id > 0 => {} // Valid positive integer
                    _ => {
                        return Err(AppError::ValidationError(
                            "AniList ID must be a positive integer".to_string(),
                        ))
                    }
                }
            }
            _ => {
                // For other providers, just check it's not empty or "0"
                // Could be extended with provider-specific rules later
            }
        }

        Ok(())
    }

    /// Check if external ID is considered valid (not empty, not "0")
    pub fn is_valid_external_id(external_id: &str) -> bool {
        !external_id.is_empty() && external_id != "0"
    }

    pub fn validate_pagination(offset: i64, limit: i64) -> Result<(), AppError> {
        if offset < 0 {
            return Err(AppError::ValidationError(
                "Offset cannot be negative".to_string(),
            ));
        }
        if limit <= 0 {
            return Err(AppError::ValidationError(
                "Limit must be positive".to_string(),
            ));
        }
        if limit > 100 {
            return Err(AppError::ValidationError(
                "Limit cannot exceed 100".to_string(),
            ));
        }
        Ok(())
    }
}
