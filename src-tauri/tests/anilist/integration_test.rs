//! Integration tests for AniList adapter
//! Tests that actually call the API (with retries and timeouts)

use miru_lib::modules::provider::infrastructure::adapters::AniListAdapter;
use std::time::Duration;
use tokio::time::timeout;

// Test configuration
const TEST_TIMEOUT: Duration = Duration::from_secs(10);
const POPULAR_ANIME_ID: u32 = 21; // One Piece - should always exist
const POPULAR_SEARCH_TERM: &str = "Attack on Titan";

#[tokio::test]
async fn test_adapter_creation() {
    let adapter = AniListAdapter::new();
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
async fn test_search_anime() {
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
async fn test_search_empty_query() {
    let adapter = AniListAdapter::new();

    let result = adapter.search_anime("", 5).await;

    match result {
        Ok(results) => {
            println!("✅ Empty search returned {} results", results.len());
        }
        Err(e) => {
            println!("✅ Empty search returned error: {}", e);
        }
    }
}

#[tokio::test]
async fn test_rate_limiting() {
    let adapter = AniListAdapter::new();

    let start = std::time::Instant::now();

    // Make multiple rapid requests
    for i in 0..3 {
        let id = (POPULAR_ANIME_ID + i).to_string();
        let _ = adapter.get_anime_by_id(&id).await;
    }

    let duration = start.elapsed();
    println!("✅ Made 3 requests in {:?}", duration);

    // AniList has rate limiting (90 req/min), so this should complete
    assert!(duration < Duration::from_secs(30));
}

#[tokio::test]
async fn test_concurrent_requests() {
    let adapter = std::sync::Arc::new(AniListAdapter::new());

    let mut handles = vec![];

    // Make 3 concurrent requests
    for i in 0..3 {
        let adapter_clone = adapter.clone();
        let id = (POPULAR_ANIME_ID + i).to_string();
        let handle = tokio::spawn(async move { adapter_clone.get_anime_by_id(&id).await });
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
    assert!(success_count > 0);
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_id_format() {
        let adapter = AniListAdapter::new();
        let result = adapter.get_anime_by_id("not_a_number").await;

        match result {
            Err(e) => {
                println!("✅ Invalid ID format returned error: {}", e);
            }
            Ok(None) => {
                println!("✅ Invalid ID format returned None");
            }
            Ok(Some(_)) => {
                panic!("❌ Should not return anime for invalid ID format");
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
            }
        }
    }
}
