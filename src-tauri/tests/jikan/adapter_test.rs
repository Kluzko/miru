use miru_lib::modules::provider::{AnimeProvider, JikanAdapter, ProviderAdapter};

#[test]
fn test_adapter_creation() {
    let adapter = JikanAdapter::new();
    assert_eq!(adapter.get_provider_type(), AnimeProvider::Jikan);
}

#[test]
fn test_custom_rate_limit() {
    let adapter = JikanAdapter::with_rate_limit(5.0, 10);
    assert_eq!(adapter.get_provider_type(), AnimeProvider::Jikan);
}

#[test]
fn test_can_make_request() {
    let adapter = JikanAdapter::new();
    let _can_request = adapter.can_make_request_now();
}

#[test]
fn test_multiple_adapters() {
    let adapter1 = JikanAdapter::new();
    let adapter2 = JikanAdapter::with_rate_limit(1.0, 3);
    assert_eq!(adapter1.get_provider_type(), adapter2.get_provider_type());
}

#[test]
fn test_different_configurations() {
    let configs = vec![(1.0, 1), (2.0, 5), (10.0, 20)];
    for (rate, burst) in configs {
        let adapter = JikanAdapter::with_rate_limit(rate, burst);
        assert_eq!(adapter.get_provider_type(), AnimeProvider::Jikan);
    }
}

#[test]
fn test_adapter_consistency() {
    let adapter1 = JikanAdapter::new();
    let adapter2 = JikanAdapter::new();
    assert_eq!(adapter1.get_provider_type(), adapter2.get_provider_type());
}

#[tokio::test]
async fn test_async_operations() {
    let adapter = JikanAdapter::with_rate_limit(100.0, 50);
    assert_eq!(adapter.get_provider_type(), AnimeProvider::Jikan);
    for _ in 0..5 {
        let _can_request = adapter.can_make_request_now();
    }
}
