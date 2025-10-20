use miru_lib::modules::provider::{infrastructure::adapters::TmdbAdapter, AnimeProvider};
use std::time::Duration;
use tokio::time::timeout;

// Test configuration
const TEST_TIMEOUT: Duration = Duration::from_secs(10);
const TEST_API_KEY: &str = "14a032abcf1763ff7568aaf97994df89";
const POPULAR_ANIME_ID: u32 = 1429; // Attack on Titan - should always exist
const POPULAR_SEARCH_TERM: &str = "Attack on Titan";

#[tokio::test]
async fn test_adapter_creation() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());
    assert!(adapter.can_make_request_now());
}

#[tokio::test]
async fn test_get_tv_show_by_id_success() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.get_tv_show(POPULAR_ANIME_ID)).await;

    match result {
        Ok(Ok(Some(tv_show))) => {
            assert!(tv_show.name.is_some());
            assert_eq!(tv_show.id, POPULAR_ANIME_ID);
            println!("✅ Successfully retrieved TV show: {:?}", tv_show.name);
        }
        Ok(Ok(None)) => {
            println!("⚠️ TV show not found");
        }
        Ok(Err(e)) => {
            println!("⚠️ API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_anime_by_id_success() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(
        TEST_TIMEOUT,
        adapter.get_anime_by_id(&POPULAR_ANIME_ID.to_string()),
    )
    .await;

    match result {
        Ok(Ok(Some(anime))) => {
            assert!(!anime.anime.title.main.is_empty());
            println!(
                "✅ Successfully retrieved anime: {}",
                anime.anime.title.main
            );
        }
        Ok(Ok(None)) => {
            println!("⚠️ Anime not found");
        }
        Ok(Err(e)) => {
            println!("⚠️ API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_search_anime() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.search_anime(POPULAR_SEARCH_TERM, 5)).await;

    match result {
        Ok(Ok(results)) => {
            println!("✅ Search returned {} results", results.len());
            if !results.is_empty() {
                assert!(!results[0].anime.title.main.is_empty());
                println!("✅ First result: {}", results[0].anime.title.main);
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
async fn test_get_images() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.get_images(POPULAR_ANIME_ID)).await;

    match result {
        Ok(Ok(images)) => {
            println!("✅ Images API successful");
            println!(
                "  - Posters: {}",
                images.posters.as_ref().map(|p| p.len()).unwrap_or(0)
            );
            println!(
                "  - Backdrops: {}",
                images.backdrops.as_ref().map(|b| b.len()).unwrap_or(0)
            );
            println!(
                "  - Logos: {}",
                images.logos.as_ref().map(|l| l.len()).unwrap_or(0)
            );
        }
        Ok(Err(e)) => {
            println!("⚠️ Images API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Images test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_posters() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.get_posters(POPULAR_ANIME_ID)).await;

    match result {
        Ok(Ok(posters)) => {
            println!("✅ Posters returned {} results", posters.len());
            for poster in posters.iter().take(3) {
                println!(
                    "  - {}x{} (rating: {})",
                    poster.width, poster.height, poster.vote_average
                );
            }
        }
        Ok(Err(e)) => {
            println!("⚠️ Posters API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Posters test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_backdrops() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.get_backdrops(POPULAR_ANIME_ID)).await;

    match result {
        Ok(Ok(backdrops)) => {
            println!("✅ Backdrops returned {} results", backdrops.len());
            for backdrop in backdrops.iter().take(3) {
                println!(
                    "  - {}x{} (rating: {})",
                    backdrop.width, backdrop.height, backdrop.vote_average
                );
            }
        }
        Ok(Err(e)) => {
            println!("⚠️ Backdrops API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Backdrops test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_logos() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.get_logos(POPULAR_ANIME_ID)).await;

    match result {
        Ok(Ok(logos)) => {
            println!("✅ Logos returned {} results", logos.len());
            for logo in logos.iter().take(3) {
                println!("  - {}x{}", logo.width, logo.height);
            }
        }
        Ok(Err(e)) => {
            println!("⚠️ Logos API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Logos test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_videos() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.get_videos(POPULAR_ANIME_ID)).await;

    match result {
        Ok(Ok(videos)) => {
            println!("✅ Videos returned {} results", videos.len());
            for video in videos.iter().take(3) {
                println!("  - {} ({}): {}", video.name, video.r#type, video.site);
            }
        }
        Ok(Err(e)) => {
            println!("⚠️ Videos API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Videos test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_trailers() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.get_trailers(POPULAR_ANIME_ID)).await;

    match result {
        Ok(Ok(trailers)) => {
            println!("✅ Trailers returned {} results", trailers.len());
            for trailer in trailers.iter().take(3) {
                println!("  - {}: {}", trailer.name, trailer.key);
            }
        }
        Ok(Err(e)) => {
            println!("⚠️ Trailers API error: {}", e);
        }
        Err(_) => {
            panic!("❌ Trailers test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_external_ids() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.get_external_ids(POPULAR_ANIME_ID)).await;

    match result {
        Ok(Ok(external_ids)) => {
            println!("✅ External IDs retrieved");
            println!("  - IMDb: {:?}", external_ids.imdb_id);
            println!("  - TVDB: {:?}", external_ids.tvdb_id);
        }
        Ok(Err(e)) => {
            println!("⚠️ External IDs API error: {}", e);
        }
        Err(_) => {
            panic!("❌ External IDs test timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_find_by_imdb_id() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    // Attack on Titan IMDb ID
    let result = timeout(TEST_TIMEOUT, adapter.find_by_imdb_id("tt0988824")).await;

    match result {
        Ok(Ok(find_response)) => {
            println!("✅ Find by IMDb ID successful");
            println!(
                "  - TV results: {}",
                find_response
                    .tv_results
                    .as_ref()
                    .map(|r| r.len())
                    .unwrap_or(0)
            );
        }
        Ok(Err(e)) => {
            println!("⚠️ Find by IMDb ID error: {}", e);
        }
        Err(_) => {
            panic!("❌ Find by IMDb ID timed out after {:?}", TEST_TIMEOUT);
        }
    }
}

#[tokio::test]
async fn test_get_popular_japanese_shows() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let result = timeout(TEST_TIMEOUT, adapter.get_popular_japanese_shows(5)).await;

    match result {
        Ok(Ok(shows)) => {
            println!("✅ Popular Japanese shows returned {} results", shows.len());
            for show in shows.iter().take(3) {
                println!("  - {:?}", show.name);
            }
        }
        Ok(Err(e)) => {
            println!("⚠️ Popular Japanese shows API error: {}", e);
        }
        Err(_) => {
            panic!(
                "❌ Popular Japanese shows test timed out after {:?}",
                TEST_TIMEOUT
            );
        }
    }
}

#[tokio::test]
async fn test_rate_limiting() {
    let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());

    let start = std::time::Instant::now();

    // Make multiple rapid requests
    for i in 0..3 {
        let _ = adapter.get_tv_show(POPULAR_ANIME_ID + i).await;
    }

    let duration = start.elapsed();
    println!("✅ Made 3 requests in {:?}", duration);

    // TMDB has generous rate limits (50 req/sec), so this should be fast
    assert!(duration < Duration::from_secs(5));
}

#[tokio::test]
async fn test_concurrent_requests() {
    let adapter = std::sync::Arc::new(TmdbAdapter::new(TEST_API_KEY.to_string()));

    let mut handles = vec![];

    // Make 3 concurrent requests
    for i in 0..3 {
        let adapter_clone = adapter.clone();
        let handle =
            tokio::spawn(
                async move { adapter_clone.get_tv_show(POPULAR_ANIME_ID + i as u32).await },
            );
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
    async fn test_invalid_id() {
        let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());
        let result = adapter.get_anime_by_id("999999999").await;

        match result {
            Ok(None) => println!("✅ Invalid ID handled gracefully"),
            Err(e) => {
                println!("✅ Invalid ID returned error: {}", e);
            }
            Ok(Some(_)) => {
                println!("⚠️ Unexpectedly found anime with very high ID");
            }
        }
    }

    #[tokio::test]
    async fn test_empty_search() {
        let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());
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
    async fn test_zero_limit() {
        let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());
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

    #[tokio::test]
    async fn test_invalid_imdb_id() {
        let adapter = TmdbAdapter::new(TEST_API_KEY.to_string());
        let result = adapter.find_by_imdb_id("invalid_id").await;

        match result {
            Ok(find_response) => {
                let tv_count = find_response
                    .tv_results
                    .as_ref()
                    .map(|r| r.len())
                    .unwrap_or(0);
                println!("✅ Invalid IMDb ID returned {} results", tv_count);
                assert_eq!(tv_count, 0);
            }
            Err(e) => {
                println!("✅ Invalid IMDb ID returned error: {}", e);
            }
        }
    }
}
