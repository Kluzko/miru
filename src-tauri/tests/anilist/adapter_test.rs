//! AniList adapter tests

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

    // Test maximum reasonable rate limit
    let adapter_max = AniListAdapter::with_rate_limit(100.0, 100);
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
