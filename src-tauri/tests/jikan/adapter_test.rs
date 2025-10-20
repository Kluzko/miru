use miru_lib::modules::provider::JikanAdapter;

#[test]
fn test_adapter_creation() {
    let adapter = JikanAdapter::new();
    assert!(adapter.can_make_request_now());
}

#[test]
fn test_custom_rate_limit() {
    let adapter = JikanAdapter::with_rate_limit(5.0, 10);
    assert!(adapter.can_make_request_now());
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

    assert!(adapter1.can_make_request_now());
    assert!(adapter2.can_make_request_now());
}

#[test]
fn test_different_configurations() {
    let configs = vec![(1.0, 1), (2.0, 5), (10.0, 20)];

    for (rate, burst) in configs {
        let adapter = JikanAdapter::with_rate_limit(rate, burst);
        assert!(adapter.can_make_request_now());
    }
}

#[test]
fn test_adapter_consistency() {
    let adapter1 = JikanAdapter::new();
    let adapter2 = JikanAdapter::new();

    assert_eq!(
        adapter1.can_make_request_now(),
        adapter2.can_make_request_now()
    );
}

#[tokio::test]
async fn test_async_operations() {
    let adapter = JikanAdapter::with_rate_limit(100.0, 50);

    for _ in 0..5 {
        let _can_request = adapter.can_make_request_now();
    }
}
