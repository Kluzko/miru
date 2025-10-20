use miru_lib::modules::provider::AniListAdapter;

#[test]
fn test_adapter_creation() {
    let adapter = AniListAdapter::new();
    // Verify adapter created successfully
    assert!(adapter.can_make_request_now());
}

#[test]
fn test_custom_rate_limit() {
    let adapter = AniListAdapter::with_rate_limit(1.5, 5);
    // Verify custom rate limit adapter created successfully
    assert!(adapter.can_make_request_now());
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

    // Both adapters should be functional
    assert!(adapter1.can_make_request_now());
    assert!(adapter2.can_make_request_now());
}

#[test]
fn test_different_configurations() {
    let configs = vec![(0.5, 1), (1.5, 5), (3.0, 10)];

    for (rate, burst) in configs {
        let adapter = AniListAdapter::with_rate_limit(rate, burst);
        // Verify each configuration works
        assert!(adapter.can_make_request_now());
    }
}

#[test]
fn test_adapter_consistency() {
    let adapter1 = AniListAdapter::new();
    let adapter2 = AniListAdapter::new();

    // Both adapters should behave consistently
    assert_eq!(
        adapter1.can_make_request_now(),
        adapter2.can_make_request_now()
    );
}

#[tokio::test]
async fn test_async_operations() {
    let adapter = AniListAdapter::with_rate_limit(10.0, 20);

    for _ in 0..5 {
        let _can_request = adapter.can_make_request_now();
    }
}

#[test]
fn test_rate_limit_boundaries() {
    // Test minimum rate limit
    let adapter_min = AniListAdapter::with_rate_limit(0.1, 1);
    assert!(adapter_min.can_make_request_now());

    // Test maximum reasonable rate limit
    let adapter_max = AniListAdapter::with_rate_limit(100.0, 100);
    assert!(adapter_max.can_make_request_now());
}
