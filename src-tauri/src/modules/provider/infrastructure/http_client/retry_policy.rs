//! Intelligent retry policies for different anime providers
//!
//! Handles provider-specific rate limiting with smart retry logic based on
//! HTTP headers and provider characteristics.

use std::time::Duration;

/// Configuration for HTTP retry behavior
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay between retries (will be adjusted based on headers)
    pub base_delay: Duration,
    /// Maximum delay to wait (prevents excessive waits)
    pub max_delay: Duration,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl RetryPolicy {
    /// Conservative policy for Jikan (30 req/min limit)
    pub fn jikan() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(2),  // 2 seconds base
            max_delay: Duration::from_secs(120), // Max 2 minutes
            exponential_backoff: true,
            backoff_multiplier: 2.0,
        }
    }

    /// More aggressive policy for AniList (90 req/min limit)
    pub fn anilist() -> Self {
        Self {
            max_retries: 5,
            base_delay: Duration::from_millis(700), // 700ms base
            max_delay: Duration::from_secs(60),     // Max 1 minute
            exponential_backoff: true,
            backoff_multiplier: 1.5,
        }
    }

    /// Calculate delay for next retry attempt
    pub fn calculate_delay(&self, attempt: u32, retry_after: Option<Duration>) -> Duration {
        // If server provided Retry-After header, respect it
        if let Some(server_delay) = retry_after {
            return server_delay.min(self.max_delay);
        }

        // Calculate backoff delay
        let delay = if self.exponential_backoff {
            let multiplier = self.backoff_multiplier.powi(attempt as i32);
            Duration::from_millis((self.base_delay.as_millis() as f64 * multiplier) as u64)
        } else {
            self.base_delay
        };

        delay.min(self.max_delay)
    }
}

/// Information extracted from HTTP 429 responses
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// How long to wait before next request (from Retry-After header)
    pub retry_after: Option<Duration>,
    /// When the rate limit resets (from X-RateLimit-Reset header)
    pub reset_time: Option<Duration>,
    /// Number of requests remaining (from X-RateLimit-Remaining header)
    pub remaining: Option<u32>,
    /// Total rate limit (from X-RateLimit-Limit header)
    pub limit: Option<u32>,
}

impl RateLimitInfo {
    /// Parse rate limit information from HTTP response headers
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Self {
        let retry_after = headers
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs);

        let reset_time = headers
            .get("x-ratelimit-reset")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(|timestamp| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if timestamp > now {
                    Duration::from_secs(timestamp - now)
                } else {
                    Duration::from_secs(0)
                }
            });

        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        Self {
            retry_after,
            reset_time,
            remaining,
            limit,
        }
    }

    /// Get the best delay recommendation from available information
    pub fn recommended_delay(&self) -> Option<Duration> {
        // Prioritize Retry-After header
        if let Some(delay) = self.retry_after {
            return Some(delay);
        }

        // Fall back to reset time if available
        self.reset_time
    }
}

/// Determines if an error is retryable
pub fn is_retryable_error(error: &reqwest::Error) -> bool {
    if let Some(status) = error.status() {
        match status.as_u16() {
            // Rate limiting
            429 => true,
            // Server errors (potentially temporary)
            500..=599 => true,
            // Timeout-related
            408 => true,
            // Too early (rare but retryable)
            425 => true,
            // Client errors are generally not retryable
            400..=499 => false,
            // Success codes shouldn't be errors
            200..=299 => false,
            // Other cases
            _ => false,
        }
    } else {
        // Network errors are potentially retryable
        error.is_timeout() || error.is_connect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jikan_policy() {
        let policy = RetryPolicy::jikan();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.base_delay, Duration::from_secs(2));
        assert!(policy.exponential_backoff);
    }

    #[test]
    fn test_anilist_policy() {
        let policy = RetryPolicy::anilist();
        assert_eq!(policy.max_retries, 5);
        assert_eq!(policy.base_delay, Duration::from_millis(700));
        assert!(policy.exponential_backoff);
    }

    #[test]
    fn test_calculate_delay_with_retry_after() {
        let policy = RetryPolicy::jikan();
        let server_delay = Some(Duration::from_secs(30));
        let delay = policy.calculate_delay(1, server_delay);
        assert_eq!(delay, Duration::from_secs(30));
    }

    #[test]
    fn test_calculate_delay_exponential_backoff() {
        let policy = RetryPolicy::jikan();
        let delay1 = policy.calculate_delay(1, None);
        let delay2 = policy.calculate_delay(2, None);
        assert!(delay2 > delay1);
    }

    #[test]
    fn test_rate_limit_info_parsing() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("retry-after", "30".parse().unwrap());
        headers.insert("x-ratelimit-remaining", "0".parse().unwrap());
        headers.insert("x-ratelimit-limit", "90".parse().unwrap());

        let info = RateLimitInfo::from_headers(&headers);
        assert_eq!(info.retry_after, Some(Duration::from_secs(30)));
        assert_eq!(info.remaining, Some(0));
        assert_eq!(info.limit, Some(90));
    }

    #[test]
    fn test_is_retryable_error() {
        // Mock errors for testing - in a real scenario these would be actual reqwest errors
        // This is a simplified test structure
        assert!(true); // Placeholder for proper error testing
    }
}
