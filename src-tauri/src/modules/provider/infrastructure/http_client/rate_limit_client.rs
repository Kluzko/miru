//! Intelligent HTTP client with automatic rate limiting and retry logic
//!
//! This client eliminates code duplication across providers and handles
//! rate limiting intelligently based on HTTP headers and provider policies.

use super::retry_policy::{is_retryable_error, RateLimitInfo, RetryPolicy};
use crate::shared::errors::{AppError, AppResult};
use governor::{Quota, RateLimiter as GovernorRateLimiter};
use reqwest::{Client, Method, Response};
use serde_json::Value;
use std::num::NonZeroU32;
use std::time::Duration;
use tokio::time::sleep;

/// Intelligent HTTP client that handles rate limiting and retries
pub struct RateLimitClient {
    client: Client,
    rate_limiter: GovernorRateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
        governor::middleware::NoOpMiddleware,
    >,
    retry_policy: RetryPolicy,
    user_agent: String,
    provider_name: String,
}

impl RateLimitClient {
    /// Create a new client for Jikan API
    pub fn for_jikan() -> Self {
        Self::new(
            "Jikan",
            RetryPolicy::jikan(),
            // Jikan v4: ~60 req/min = 1.0 req/sec average with 3 req/sec burst capability
            Self::create_rate_limiter(1.0, 3),
            "miru/1.0 (https://github.com/your-repo/miru)".to_string(),
        )
    }

    /// Create a new client for AniList API
    pub fn for_anilist() -> Self {
        Self::new(
            "AniList",
            RetryPolicy::anilist(),
            // AniList: 30 req/min (degraded state) = 0.5 req/sec
            Self::create_rate_limiter(0.5, 2),
            "miru/1.0 (https://github.com/your-repo/miru)".to_string(),
        )
    }

    /// Create a rate limiter with specified requests per second and burst capacity
    fn create_rate_limiter(
        requests_per_second: f64,
        burst_size: u32,
    ) -> GovernorRateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
        governor::middleware::NoOpMiddleware,
    > {
        // Convert rate to duration between requests
        let duration = if requests_per_second > 0.0 {
            Duration::from_secs_f64(1.0 / requests_per_second)
        } else {
            Duration::MAX // Effectively disable if rate is 0
        };

        let burst = NonZeroU32::new(burst_size.max(1)).unwrap();
        let quota = Quota::with_period(duration).unwrap().allow_burst(burst);

        GovernorRateLimiter::direct(quota)
    }

    /// Create a custom client
    pub fn new(
        provider_name: &str,
        retry_policy: RetryPolicy,
        rate_limiter: GovernorRateLimiter<
            governor::state::direct::NotKeyed,
            governor::state::InMemoryState,
            governor::clock::DefaultClock,
            governor::middleware::NoOpMiddleware,
        >,
        user_agent: String,
    ) -> Self {
        Self {
            client: Client::new(),
            rate_limiter,
            retry_policy,
            user_agent,
            provider_name: provider_name.to_string(),
        }
    }

    /// Make a GET request with intelligent rate limiting and retries
    pub async fn get<T>(&self, url: &str) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.request_with_retries(Method::GET, url, None).await
    }

    /// Make a POST request with JSON body
    pub async fn post_json<T>(&self, url: &str, body: &Value) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.request_with_retries(Method::POST, url, Some(body.clone()))
            .await
    }

    /// Make a request with automatic retries and rate limiting
    async fn request_with_retries<T>(
        &self,
        method: Method,
        url: &str,
        body: Option<Value>,
    ) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut last_error = None;

        for attempt in 0..=self.retry_policy.max_retries {
            // Wait for rate limiter before attempting request
            self.rate_limiter.until_ready().await;

            // Build and send request
            match self.build_and_send_request(&method, url, &body).await {
                Ok(response) => {
                    // Check for rate limiting
                    if response.status() == 429 {
                        let rate_limit_info = RateLimitInfo::from_headers(response.headers());

                        if attempt < self.retry_policy.max_retries {
                            let delay = self.calculate_retry_delay(attempt, &rate_limit_info);
                            log::warn!(
                                "{} API rate limited (attempt {}/{}). Waiting {:?} before retry.",
                                self.provider_name,
                                attempt + 1,
                                self.retry_policy.max_retries + 1,
                                delay
                            );
                            sleep(delay).await;
                            continue;
                        } else {
                            return Err(AppError::ApiError(format!(
                                "{} API rate limit exceeded after {} attempts",
                                self.provider_name,
                                self.retry_policy.max_retries + 1
                            )));
                        }
                    }

                    // Handle other HTTP errors
                    if !response.status().is_success() {
                        let error_msg = format!(
                            "{} API returned error: {}",
                            self.provider_name,
                            response.status()
                        );

                        // Only retry server errors
                        if response.status().is_server_error()
                            && attempt < self.retry_policy.max_retries
                        {
                            let delay = self.retry_policy.calculate_delay(attempt, None);
                            log::warn!(
                                "{} (attempt {}/{}). Retrying in {:?}",
                                error_msg,
                                attempt + 1,
                                self.retry_policy.max_retries + 1,
                                delay
                            );
                            sleep(delay).await;
                            continue;
                        } else {
                            return Err(AppError::ApiError(error_msg));
                        }
                    }

                    // Parse successful response
                    return self.parse_response(response).await;
                }
                Err(e) => {
                    last_error = Some(AppError::ApiError(e.to_string()));

                    // Only retry if error is retryable and we haven't exceeded max attempts
                    if is_retryable_error(&e) && attempt < self.retry_policy.max_retries {
                        let delay = self.retry_policy.calculate_delay(attempt, None);
                        log::warn!(
                            "{} API request failed (attempt {}/{}): {}. Retrying in {:?}",
                            self.provider_name,
                            attempt + 1,
                            self.retry_policy.max_retries + 1,
                            e,
                            delay
                        );
                        sleep(delay).await;
                        continue;
                    } else {
                        return Err(AppError::ApiError(format!(
                            "{} API request failed: {}",
                            self.provider_name, e
                        )));
                    }
                }
            }
        }

        // If we get here, all retries were exhausted
        Err(AppError::ApiError(format!(
            "{} API request failed after {} attempts: {}",
            self.provider_name,
            self.retry_policy.max_retries + 1,
            last_error.map_or_else(|| "Unknown error".to_string(), |e| e.to_string())
        )))
    }

    /// Build and send the actual HTTP request
    async fn build_and_send_request(
        &self,
        method: &Method,
        url: &str,
        body: &Option<Value>,
    ) -> Result<Response, reqwest::Error> {
        let mut request_builder = self
            .client
            .request(method.clone(), url)
            .header("User-Agent", &self.user_agent);

        // Add provider-specific headers
        match self.provider_name.as_str() {
            "AniList" => {
                request_builder = request_builder
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json");
            }
            "Jikan" => {
                // Jikan typically doesn't need special headers for GET requests
            }
            _ => {
                // Default headers for unknown providers
                request_builder = request_builder.header("Accept", "application/json");
            }
        }

        // Add body for POST requests
        if let Some(json_body) = body {
            request_builder = request_builder.json(json_body);
        }

        request_builder.send().await
    }

    /// Parse the response based on provider expectations
    async fn parse_response<T>(&self, response: Response) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response_text = response.text().await.map_err(|e| {
            AppError::SerializationError(format!(
                "Failed to read {} response: {}",
                self.provider_name, e
            ))
        })?;

        // Parse JSON response
        serde_json::from_str(&response_text).map_err(|e| {
            AppError::SerializationError(format!(
                "Failed to parse {} response: {}. Response: {}",
                self.provider_name,
                e,
                if response_text.len() > 200 {
                    format!("{}...", &response_text[..200])
                } else {
                    response_text
                }
            ))
        })
    }

    /// Calculate delay for retry based on rate limit info and policy
    fn calculate_retry_delay(&self, attempt: u32, rate_limit_info: &RateLimitInfo) -> Duration {
        // Use server-provided delay if available
        if let Some(server_delay) = rate_limit_info.recommended_delay() {
            return server_delay.min(self.retry_policy.max_delay);
        }

        // Fall back to policy-based delay
        self.retry_policy.calculate_delay(attempt, None)
    }

    /// Check if a request can be made now (for testing/debugging)
    pub fn can_make_request_now(&self) -> bool {
        self.rate_limiter.check().is_ok()
    }

    /// Get provider name
    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let jikan_client = RateLimitClient::for_jikan();
        assert_eq!(jikan_client.provider_name(), "Jikan");

        let anilist_client = RateLimitClient::for_anilist();
        assert_eq!(anilist_client.provider_name(), "AniList");
    }

    #[test]
    fn test_can_make_request() {
        let client = RateLimitClient::for_jikan();
        assert!(client.can_make_request_now());
    }

    // Integration tests would require actual API calls
    // and should be in a separate test suite
}
