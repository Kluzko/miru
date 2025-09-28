//! Rate limiter tests
//!
//! Tests the HTTP client rate limiting implementation with RateLimitClient.

use miru_lib::modules::provider::infrastructure::http_client::RateLimitClient;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_rate_limit_client_creation() {
    let client = RateLimitClient::for_jikan();
    assert_eq!(client.provider_name(), "Jikan");
}

#[tokio::test]
async fn test_can_make_request() {
    let client = RateLimitClient::for_anilist();
    assert!(client.can_make_request_now());
}

#[tokio::test]
async fn test_multiple_clients() {
    let jikan_client = RateLimitClient::for_jikan();
    let anilist_client = RateLimitClient::for_anilist();

    assert_eq!(jikan_client.provider_name(), "Jikan");
    assert_eq!(anilist_client.provider_name(), "AniList");

    assert!(jikan_client.can_make_request_now());
    assert!(anilist_client.can_make_request_now());
}
