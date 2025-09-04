use regex::Regex;

use crate::shared::errors::AppError;

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
        let re = Regex::new(r"^[a-zA-Z0-9\s\-_]+$").unwrap();
        if !re.is_match(name) {
            return Err(AppError::ValidationError(
                "Collection name contains invalid characters".to_string(),
            ));
        }
        Ok(())
    }

    pub fn validate_mal_id(mal_id: i32) -> Result<(), AppError> {
        if mal_id <= 0 {
            return Err(AppError::ValidationError(
                "MAL ID must be positive".to_string(),
            ));
        }
        Ok(())
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
