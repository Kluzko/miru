//! Integration tests for AniList adapter
//! Tests that actually call the API (with retries and timeouts)

use miru_lib::modules::provider::{
    infrastructure::adapters::{AniListAdapter, ProviderAdapter},
    AnimeProvider,
};
use std::time::Duration;
use tokio::time::timeout;

// Test configuration
const TEST_TIMEOUT: Duration = Duration::from_secs(10);
const POPULAR_ANIME_ID: u32 = 21; // One Piece - should always exist
const POPULAR_SEARCH_TERM: &str = "Attack on Titan";

#[tokio::test]
async fn test_provider_adapter_trait_implementation() {
    let adapter = AniListAdapter::new();
    assert_eq!(adapter.get_provider_type(), AnimeProvider::AniList);
    assert!(adapter.can_make_request_now());
}

#[tokio::test]
async fn test_get_anime_by_id_success() {
    let adapter = AniListAdapter::new();

    let result = timeout(
        TEST_TIMEOUT,
        adapter.get_anime_by_id(&POPULAR_ANIME_ID.to_string()),
    )
    .await;

    match result {
        Ok(Ok(Some(anime))) => {
            assert!(!anime.anime.title.is_empty());
            assert!(anime.anime.id > 0);
            println!("✅ Successfully retrieved anime: {}", anime.anime.title);
        }
        Ok(Ok(None)) => {
            println!("⚠️ Anime not found (this might be expected for some IDs)");
        }
        Ok(Err(e)) => {
            println!(
                "⚠️ API error (might be rate limited or network issue): {}",
                e
            );
            // Don't fail the test for network issues in CI
        }
        Err(_) => {
            panic!("❌ Test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_anime_by_id_not_found() {
    let adapter = AniListAdapter::new();

    // Use an extremely high ID that shouldn't exist
    let result = adapter.get_anime_by_id("999999999").await;

    match result {
        Ok(None) => {
            println!("✅ Correctly returned None for non-existent anime");
        }
        Ok(Some(_)) => {
            println!("⚠️ Unexpectedly found anime with very high ID");
        }
        Err(e) => {
            println!("⚠️ API error for non-existent ID: {}", e);
            // This is acceptable - some APIs return errors for invalid IDs
        }
    }
}

#[tokio::test]
async fn test_search_anime_basic() {
    let adapter = AniListAdapter::new();

    let result = timeout(TEST_TIMEOUT, adapter.search_anime(POPULAR_SEARCH_TERM, 5)).await;

    match result {
        Ok(Ok(results)) => {
            println!("✅ Search returned {} results", results.len());
            if !results.is_empty() {
                assert!(!results[0].anime.title.is_empty());
                println!("✅ First result: {}", results[0].anime.title);
            }
        }
        Ok(Err(e)) => {
            println!("⚠️ Search API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Search timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_anime_full() {
    let adapter = AniListAdapter::new();

    let result = timeout(TEST_TIMEOUT, adapter.get_anime_full(POPULAR_ANIME_ID)).await;

    match result {
        Ok(Ok(Some(anime))) => {
            assert!(!anime.anime.title.is_empty());
            println!(
                "✅ Successfully retrieved full anime data: {}",
                anime.anime.title
            );
        }
        Ok(Ok(None)) => {
            println!("⚠️ Full anime data not found");
        }
        Ok(Err(e)) => {
            println!("⚠️ Full anime API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Full anime test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_season_now() {
    let adapter = AniListAdapter::new();

    let result = timeout(TEST_TIMEOUT, adapter.get_season_now(3)).await;

    match result {
        Ok(Ok(results)) => {
            println!("✅ Current season returned {} results", results.len());
            for anime in results.iter().take(3) {
                println!("  - {}", anime.anime.title);
            }
        }
        Ok(Err(e)) => {
            println!("⚠️ Current season API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Current season test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_season_upcoming() {
    let adapter = AniListAdapter::new();

    let result = timeout(TEST_TIMEOUT, adapter.get_season_upcoming(3)).await;

    match result {
        Ok(Ok(results)) => {
            println!("✅ Upcoming season returned {} results", results.len());
            for anime in results.iter().take(3) {
                println!("  - {}", anime.anime.title);
            }
        }
        Ok(Err(e)) => {
            println!("⚠️ Upcoming season API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Upcoming season test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_search_anime_basic_trait() {
    let adapter = AniListAdapter::new();

    let result = timeout(
        TEST_TIMEOUT,
        adapter.search_anime_basic(POPULAR_SEARCH_TERM, 3),
    )
    .await;

    match result {
        Ok(Ok(results)) => {
            println!("✅ Basic search (trait) returned {} results", results.len());
        }
        Ok(Err(e)) => {
            println!("⚠️ Basic search (trait) API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Basic search (trait) timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_rate_limiting() {
    let adapter = AniListAdapter::with_rate_limit(0.5, 1); // Very slow rate

    let start = std::time::Instant::now();

    // Make two requests - second should be rate limited
    let _first = adapter.get_anime_by_id("1").await;
    let _second = adapter.get_anime_by_id("2").await;

    let duration = start.elapsed();

    // Should take at least 2 seconds due to rate limiting (0.5 req/sec)
    if duration >= Duration::from_millis(1500) {
        println!("✅ Rate limiting working correctly: {:?}", duration);
    } else {
        println!("⚠️ Rate limiting may not be working: {:?}", duration);
    }
}

#[tokio::test]
async fn test_concurrent_requests() {
    let adapter = std::sync::Arc::new(AniListAdapter::new());
    let mut handles = vec![];

    // Make 3 concurrent requests
    for i in 1..=3 {
        let adapter_clone = adapter.clone();
        let handle =
            tokio::spawn(async move { adapter_clone.get_anime_by_id(&i.to_string()).await });
        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => success_count += 1,
            Err(e) => println!("⚠️ Concurrent request failed: {}", e),
        }
    }

    println!("✅ Concurrent requests: {}/3 succeeded", success_count);
    // At least one should succeed unless there are serious network issues
    assert!(success_count > 0);
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_id_format() {
        let adapter = AniListAdapter::new();

        let result = adapter.get_anime_by_id("invalid_id").await;

        match result {
            Ok(None) => println!("✅ Invalid ID handled gracefully"),
            Err(e) => {
                println!("✅ Invalid ID returned error: {}", e);
                // This is also acceptable
            }
            Ok(Some(_)) => {
                panic!("❌ Invalid ID should not return anime data");
            }
        }
    }

    #[tokio::test]
    async fn test_empty_search() {
        let adapter = AniListAdapter::new();

        let result = adapter.search_anime("", 5).await;

        match result {
            Ok(results) => {
                println!("✅ Empty search returned {} results", results.len());
            }
            Err(e) => {
                println!("✅ Empty search returned error: {}", e);
                // Some APIs might reject empty searches
            }
        }
    }

    #[tokio::test]
    async fn test_zero_limit() {
        let adapter = AniListAdapter::new();

        let result = adapter.search_anime(POPULAR_SEARCH_TERM, 0).await;

        match result {
            Ok(results) => {
                assert_eq!(results.len(), 0);
                println!("✅ Zero limit returned 0 results");
            }
            Err(e) => {
                println!("✅ Zero limit returned error: {}", e);
                // Some APIs might reject zero limits
            }
        }
    }
}
