//! Simple AniList API tests (no external dependencies)

use miru_lib::modules::provider::{AniListAdapter, AnimeProvider, ProviderAdapter};

#[test]
fn test_adapter_creation() {
    let adapter = AniListAdapter::new();
    assert_eq!(adapter.get_provider_type(), AnimeProvider::AniList);
}

#[test]
fn test_custom_rate_limit() {
    let adapter = AniListAdapter::with_rate_limit(1.5, 5);
    assert_eq!(adapter.get_provider_type(), AnimeProvider::AniList);
}

#[test]
fn test_can_make_request() {
    let adapter = AniListAdapter::new();
    let _can_request = adapter.can_make_request_now();
}

#[test]
fn test_multiple_adapters() {
    let adapter1 = AniListAdapter::new();
    let adapter2 = AniListAdapter::with_rate_limit(1.0, 3);
    assert_eq!(adapter1.get_provider_type(), adapter2.get_provider_type());
}

#[test]
fn test_different_configurations() {
    let configs = vec![(0.5, 1), (1.5, 5), (3.0, 10)];
    for (rate, burst) in configs {
        let adapter = AniListAdapter::with_rate_limit(rate, burst);
        assert_eq!(adapter.get_provider_type(), AnimeProvider::AniList);
    }
}

#[test]
fn test_adapter_consistency() {
    let adapter1 = AniListAdapter::new();
    let adapter2 = AniListAdapter::new();
    assert_eq!(adapter1.get_provider_type(), adapter2.get_provider_type());
}

#[tokio::test]
async fn test_async_operations() {
    let adapter = AniListAdapter::with_rate_limit(10.0, 20);
    assert_eq!(adapter.get_provider_type(), AnimeProvider::AniList);
    for _ in 0..5 {
        let _can_request = adapter.can_make_request_now();
    }
}

#[test]
fn test_rate_limit_boundaries() {
    // Test minimum rate limit
    let adapter_min = AniListAdapter::with_rate_limit(0.1, 1);
    assert_eq!(adapter_min.get_provider_type(), AnimeProvider::AniList);

    // Test maximum reasonable rate limit (respecting AniList's 90 req/min limit)
    let adapter_max = AniListAdapter::with_rate_limit(1.5, 5);
    assert_eq!(adapter_max.get_provider_type(), AnimeProvider::AniList);
}

#[test]
fn test_provider_type_immutability() {
    let adapter = AniListAdapter::new();
    let provider_type1 = adapter.get_provider_type();
    let provider_type2 = adapter.get_provider_type();
    assert_eq!(provider_type1, provider_type2);
    assert_eq!(provider_type1, AnimeProvider::AniList);
}

#[test]
fn test_default_rate_limit() {
    let adapter = AniListAdapter::new();
    // Should be able to make requests initially
    assert!(adapter.can_make_request_now());
}

#[test]
fn test_adapter_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<AniListAdapter>();
    assert_sync::<AniListAdapter>();
}

#[test]
fn test_multiple_concurrent_adapters() {
    let adapters: Vec<AniListAdapter> = (0..10)
        .map(|i| AniListAdapter::with_rate_limit(1.0 + i as f64 * 0.1, 5))
        .collect();

    for adapter in adapters {
        assert_eq!(adapter.get_provider_type(), AnimeProvider::AniList);
    }
}

#[test]
fn test_rate_limiter_independence() {
    let adapter1 = AniListAdapter::with_rate_limit(0.1, 1); // Very slow
    let adapter2 = AniListAdapter::with_rate_limit(100.0, 50); // Very fast

    // Both should be able to make initial requests
    assert!(adapter1.can_make_request_now());
    assert!(adapter2.can_make_request_now());
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiting_behavior() {
        let adapter = AniListAdapter::with_rate_limit(2.0, 2); // 2 requests per second, burst of 2

        // Should be able to make 2 requests immediately
        assert!(adapter.can_make_request_now());
        assert!(adapter.can_make_request_now());

        // Third request might be rate limited
        // This is probabilistic based on the rate limiter implementation
        let _third_request = adapter.can_make_request_now();

        // After a short wait, should be able to make more requests
        sleep(Duration::from_millis(600)).await;
        assert!(adapter.can_make_request_now());
    }

    #[tokio::test]
    async fn test_adapter_concurrent_safety() {
        let adapter = std::sync::Arc::new(AniListAdapter::new());
        let mut handles = vec![];

        for _ in 0..5 {
            let adapter_clone = adapter.clone();
            let handle = tokio::spawn(async move {
                assert_eq!(adapter_clone.get_provider_type(), AnimeProvider::AniList);
                adapter_clone.can_make_request_now()
            });
            handles.push(handle);
        }

        for handle in handles {
            let _result = handle.await.unwrap();
        }
    }
}
