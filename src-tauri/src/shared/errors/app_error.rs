use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => {
                AppError::NotFound("Record not found in database".to_string())
            }
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

impl From<diesel::r2d2::PoolError> for AppError {
    fn from(err: diesel::r2d2::PoolError) -> Self {
        AppError::DatabaseError(format!("Database pool error: {}", err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::ExternalServiceError("Request timeout".to_string())
        } else if err.is_connect() {
            AppError::ExternalServiceError("Failed to connect to external service".to_string())
        } else if let Some(status) = err.status() {
            match status.as_u16() {
                429 => AppError::RateLimitError("Too many requests".to_string()),
                404 => AppError::NotFound("External resource not found".to_string()),
                401 | 403 => {
                    AppError::Unauthorized("Not authorized to access external service".to_string())
                }
                _ => AppError::ApiError(format!("HTTP {}: {}", status, err)),
            }
        } else {
            AppError::ApiError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerializationError(err.to_string())
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::InvalidInput(format!("Invalid UUID: {}", err))
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::InvalidInput(format!("Invalid date/time: {}", err))
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::InvalidInput(format!("Invalid number: {}", err))
    }
}

impl From<std::num::ParseFloatError> for AppError {
    fn from(err: std::num::ParseFloatError) -> Self {
        AppError::InvalidInput(format!("Invalid decimal number: {}", err))
    }
}

// Convert AppError to a format suitable for Tauri commands
impl AppError {
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
