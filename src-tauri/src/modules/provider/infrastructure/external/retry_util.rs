use crate::shared::errors::{AppError, AppResult};
use reqwest::StatusCode;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration for external API calls
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a conservative retry config for production use
    pub fn conservative() -> Self {
        Self {
            max_retries: 2,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
            jitter: true,
        }
    }

    /// Create an aggressive retry config for development/testing
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.5,
            jitter: true,
        }
    }
}

/// Retry utility for external API calls with exponential backoff
pub struct RetryUtil;

impl RetryUtil {
    /// Execute a function with retry logic and exponential backoff
    pub async fn with_retry<F, Fut, T>(
        operation: F,
        config: &RetryConfig,
        operation_name: &str,
    ) -> AppResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = AppResult<T>>,
    {
        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!(
                            "{} succeeded on attempt {} after {} retries",
                            operation_name,
                            attempt + 1,
                            attempt
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error.clone());

                    // Check if error is retryable
                    if !Self::is_retryable_error(&error) {
                        debug!(
                            "{} failed with non-retryable error: {}",
                            operation_name, error
                        );
                        return Err(error);
                    }

                    // Don't wait after the last attempt
                    if attempt < config.max_retries {
                        let delay = Self::calculate_delay(attempt, config);
                        warn!(
                            "{} failed on attempt {} ({}), retrying in {:?}",
                            operation_name,
                            attempt + 1,
                            error,
                            delay
                        );
                        sleep(delay).await;
                    } else {
                        warn!(
                            "{} failed on final attempt {} ({}), giving up",
                            operation_name,
                            attempt + 1,
                            error
                        );
                    }
                }
            }
        }

        // Return the last error if all retries failed
        Err(last_error
            .unwrap_or_else(|| AppError::ExternalServiceError("All retries exhausted".to_string())))
    }

    /// Calculate delay for the given attempt with exponential backoff and jitter
    fn calculate_delay(attempt: u32, config: &RetryConfig) -> Duration {
        let exponential_delay =
            config.base_delay.as_millis() as f64 * config.backoff_multiplier.powi(attempt as i32);

        let mut delay = Duration::from_millis(exponential_delay as u64);

        // Cap at max delay
        if delay > config.max_delay {
            delay = config.max_delay;
        }

        // Add jitter to prevent thundering herd
        if config.jitter {
            let jitter_factor = 0.1; // 10% jitter
            let jitter_ms =
                (delay.as_millis() as f64 * jitter_factor * rand::random::<f64>()) as u64;
            delay = Duration::from_millis(delay.as_millis() as u64 + jitter_ms);
        }

        delay
    }

    /// Determine if an error should trigger a retry
    fn is_retryable_error(error: &AppError) -> bool {
        match error {
            // Network-related errors - usually temporary
            AppError::ExternalServiceError(_) => true,

            // Rate limiting - should retry with backoff
            AppError::RateLimitError(_) => true,

            // API errors - check if they're temporary
            AppError::ApiError(msg) => {
                // Don't retry on clearly permanent errors
                !msg.to_lowercase().contains("not found")
                    && !msg.to_lowercase().contains("unauthorized")
                    && !msg.to_lowercase().contains("forbidden")
                    && !msg.to_lowercase().contains("bad request")
            }

            // Don't retry validation errors or permanent failures
            AppError::ValidationError(_)
            | AppError::InvalidInput(_)
            | AppError::NotFound(_)
            | AppError::NotImplemented(_)
            | AppError::Unauthorized(_)
            | AppError::InvalidOperation(_)
            | AppError::Duplicate(_) => false,

            // Internal errors and serialization errors might be temporary
            AppError::InternalError(_) | AppError::SerializationError(_) => true,

            // Database errors might be temporary
            AppError::DatabaseError(_) => true,
        }
    }

    /// Retry specifically for HTTP requests with status code analysis
    pub async fn retry_http_request<F, Fut>(
        request_fn: F,
        config: &RetryConfig,
        operation_name: &str,
    ) -> AppResult<reqwest::Response>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        Self::with_retry(
            || async {
                match request_fn().await {
                    Ok(response) => {
                        let status = response.status();
                        if Self::is_retryable_status(status) {
                            Err(Self::status_to_app_error(status))
                        } else {
                            Ok(response)
                        }
                    }
                    Err(e) => Err(AppError::ExternalServiceError(format!(
                        "HTTP request failed: {}",
                        e
                    ))),
                }
            },
            config,
            operation_name,
        )
        .await
    }

    /// Check if HTTP status code indicates a retryable error
    fn is_retryable_status(status: StatusCode) -> bool {
        match status {
            // Server errors - often temporary
            StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT => true,

            // Rate limiting - should retry with backoff
            StatusCode::TOO_MANY_REQUESTS => true,

            // Request timeout - might succeed on retry
            StatusCode::REQUEST_TIMEOUT => true,

            // Client errors and success codes - don't retry
            _ => false,
        }
    }

    /// Convert HTTP status to appropriate AppError
    fn status_to_app_error(status: StatusCode) -> AppError {
        match status {
            StatusCode::TOO_MANY_REQUESTS => {
                AppError::RateLimitError("Rate limit exceeded".to_string())
            }
            StatusCode::NOT_FOUND => AppError::NotFound("Resource not found".to_string()),
            StatusCode::UNAUTHORIZED => AppError::ApiError("Unauthorized access".to_string()),
            StatusCode::FORBIDDEN => AppError::ApiError("Access forbidden".to_string()),
            StatusCode::BAD_REQUEST => AppError::ApiError("Bad request".to_string()),
            _ if status.is_server_error() => {
                AppError::ExternalServiceError(format!("Server error: {}", status))
            }
            _ => AppError::ApiError(format!("HTTP error: {}", status)),
        }
    }
}

/// Common HTTP response handler for all providers
/// Eliminates duplicate status handling code
pub struct CommonHttpHandler;

impl CommonHttpHandler {
    /// Handle HTTP response status codes consistently across all providers
    pub fn handle_response_status(status: StatusCode, provider_name: &str) -> AppResult<()> {
        match status {
            StatusCode::OK => Ok(()),
            StatusCode::TOO_MANY_REQUESTS => Err(AppError::RateLimitError(format!(
                "{} rate limit exceeded",
                provider_name
            ))),
            StatusCode::NOT_FOUND => Err(AppError::NotFound("Resource not found".to_string())),
            StatusCode::BAD_REQUEST => Err(AppError::ApiError(format!(
                "Bad request to {} API",
                provider_name
            ))),
            StatusCode::UNAUTHORIZED => Err(AppError::ApiError(format!(
                "Unauthorized access to {} API",
                provider_name
            ))),
            StatusCode::FORBIDDEN => Err(AppError::ApiError(format!(
                "Access forbidden to {} API",
                provider_name
            ))),
            StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::BAD_GATEWAY
            | StatusCode::GATEWAY_TIMEOUT => Err(AppError::ExternalServiceError(format!(
                "{} service unavailable",
                provider_name
            ))),
            _ => Err(AppError::ApiError(format!(
                "Unexpected status code from {}: {}",
                provider_name, status
            ))),
        }
    }

    /// Create a retry-enabled HTTP client with consistent configuration
    pub fn create_http_client(timeout_secs: u64, user_agent: &str) -> AppResult<reqwest::Client> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(user_agent)
            .build()
            .map_err(|e| {
                AppError::ExternalServiceError(format!("Failed to create HTTP client: {}", e))
            })
    }

    /// Execute HTTP request with retry logic
    pub async fn execute_with_retry<F, Fut>(
        request_fn: F,
        provider_name: &str,
        operation_name: &str,
    ) -> AppResult<reqwest::Response>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    {
        let retry_config = RetryConfig::conservative();

        RetryUtil::retry_http_request(
            request_fn,
            &retry_config,
            &format!("{} {}", provider_name, operation_name),
        )
        .await
        .and_then(|response| {
            let status = response.status();
            Self::handle_response_status(status, provider_name)?;
            Ok(response)
        })
    }
}
