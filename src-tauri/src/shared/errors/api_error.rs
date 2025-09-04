use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Rate limit exceeded, retry after {retry_after} seconds")]
    RateLimit { retry_after: u64 },

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Invalid API response: {message}")]
    InvalidResponse { message: String },

    #[error("API authentication failed")]
    AuthenticationFailed,

    #[error("API service unavailable")]
    ServiceUnavailable,
}
