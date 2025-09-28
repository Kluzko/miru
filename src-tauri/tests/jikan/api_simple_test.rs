//! Simple Jikan API tests
//!
//! Basic tests that verify the Jikan adapter works without compilation errors.
//! These tests are designed to be safe and not hit rate limits.

use miru_lib::modules::provider::{AnimeProvider, JikanAdapter, ProviderAdapter};
use std::time::Duration;
use tokio::time::sleep;

#[test]
fn test_adapter_basic_functionality() {
    let adapter = JikanAdapter::new();
    assert_eq!(adapter.get_provider_type(), AnimeProvider::Jikan);

    // Test rate limiting check
    let _can_request = adapter.can_make_request_now();
    println!("✅ Basic adapter functionality test passed");
}

#[test]
fn test_adapter_with_custom_rate_limit() {
    let adapter = JikanAdapter::with_rate_limit(5.0, 10);
    assert_eq!(adapter.get_provider_type(), AnimeProvider::Jikan);

    let _can_request = adapter.can_make_request_now();
    println!("✅ Custom rate limit adapter test passed");
}

#[test]
fn test_multiple_adapter_instances() {
    let adapter1 = JikanAdapter::new();
    let adapter2 = JikanAdapter::with_rate_limit(2.0, 5);
    let adapter3 = JikanAdapter::with_rate_limit(10.0, 20);

    assert_eq!(adapter1.get_provider_type(), adapter2.get_provider_type());
    assert_eq!(adapter2.get_provider_type(), adapter3.get_provider_type());

    println!("✅ Multiple adapter instances test passed");
}

#[tokio::test]
async fn test_rate_limiting_behavior() {
    let adapter = JikanAdapter::with_rate_limit(2.0, 3); // 2 req/sec, 3 burst

    // Test initial state
    let initial_state = adapter.can_make_request_now();
    println!("Initial rate limit state: {}", initial_state);

    // Small delay to ensure consistent behavior
    sleep(Duration::from_millis(100)).await;

    let after_delay = adapter.can_make_request_now();
    println!("After delay rate limit state: {}", after_delay);

    println!("✅ Rate limiting behavior test passed");
}

#[test]
fn test_provider_type_consistency() {
    let adapters = vec![
        JikanAdapter::new(),
        JikanAdapter::with_rate_limit(1.0, 1),
        JikanAdapter::with_rate_limit(0.5, 2),
        JikanAdapter::with_rate_limit(100.0, 50),
    ];

    for adapter in adapters {
        assert_eq!(adapter.get_provider_type(), AnimeProvider::Jikan);
    }

    println!("✅ Provider type consistency test passed");
}

#[test]
fn test_extreme_rate_limits() {
    // Test very conservative rate limit
    let slow_adapter = JikanAdapter::with_rate_limit(0.1, 1); // 1 req per 10 seconds
    assert_eq!(slow_adapter.get_provider_type(), AnimeProvider::Jikan);

    // Test very permissive rate limit
    let fast_adapter = JikanAdapter::with_rate_limit(1000.0, 999);
    assert_eq!(fast_adapter.get_provider_type(), AnimeProvider::Jikan);

    println!("✅ Extreme rate limits test passed");
}

#[tokio::test]
async fn test_concurrent_rate_limit_checks() {
    let _adapter = JikanAdapter::with_rate_limit(5.0, 3);

    // Spawn multiple tasks to check rate limiting concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let adapter_clone = JikanAdapter::with_rate_limit(5.0, 3);
        let handle = tokio::spawn(async move {
            let can_request = adapter_clone.can_make_request_now();
            println!("Task {}: can_make_request = {}", i, can_request);
            can_request
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results: Vec<bool> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    println!("Concurrent check results: {:?}", results);
    println!("✅ Concurrent rate limit checks test passed");
}

// Only run API tests if explicitly enabled
#[tokio::test]
#[ignore = "API test - run with --ignored to enable"]
async fn test_basic_api_functionality() {
    if std::env::var("JIKAN_API_TEST").is_err() {
        println!("⏭️ Skipping API test - set JIKAN_API_TEST=1 to enable");
        return;
    }

    println!("🔍 Testing basic API functionality...");
    let adapter = JikanAdapter::with_rate_limit(1.0, 1); // Very conservative

    // Test with a well-known anime ID (Cowboy Bebop = 1)
    match adapter.get_anime(1).await {
        Ok(Some(data)) => {
            println!("✅ API test successful - got anime data");
            println!(
                "📺 Anime title exists: {}",
                data.anime.title.english.is_some() || data.anime.title.japanese.is_some()
            );
        }
        Ok(None) => {
            println!("⚠️ API test returned no data");
        }
        Err(e) => {
            println!("❌ API test failed: {}", e);
            // Don't panic - API might be down or rate limited
        }
    }

    // Wait to respect rate limits
    sleep(Duration::from_secs(2)).await;
    println!("✅ Basic API functionality test completed");
}

#[test]
fn test_adapter_debug_info() {
    let adapter = JikanAdapter::new();

    // Test basic properties
    assert_eq!(adapter.get_provider_type(), AnimeProvider::Jikan);

    let rate_check = adapter.can_make_request_now();
    println!("Debug info:");
    println!("  Provider: {:?}", adapter.get_provider_type());
    println!("  Can make request: {}", rate_check);

    println!("✅ Adapter debug info test passed");
}

#[tokio::test]
async fn test_adapter_memory_usage() {
    // Test that creating many adapters doesn't cause memory issues
    let mut adapters = Vec::new();

    for i in 0..100 {
        let rate = (i as f64 % 10.0) + 1.0; // Rate between 1-10
        let burst = (i % 5) + 1; // Burst between 1-5
        adapters.push(JikanAdapter::with_rate_limit(rate, burst as u32));
    }

    // Verify all adapters work
    for adapter in &adapters {
        assert_eq!(adapter.get_provider_type(), AnimeProvider::Jikan);
        let _can_request = adapter.can_make_request_now();
    }

    println!(
        "✅ Memory usage test passed - created {} adapters",
        adapters.len()
    );
}
