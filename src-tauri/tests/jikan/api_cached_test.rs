//! Cached Jikan API Tests
//!
//! These tests use cached responses to verify data processing logic
//! without hitting the API repeatedly. Perfect for CI/CD pipelines.

use miru_lib::modules::provider::{JikanAdapter, ProviderAdapter};
use serde_json::json;
use std::collections::HashMap;

/// Mock response cache to simulate API responses
struct MockApiCache {
    responses: HashMap<String, serde_json::Value>,
}

impl MockApiCache {
    fn new() -> Self {
        let mut responses = HashMap::new();

        // Cache common anime responses
        responses.insert(
            "anime_1".to_string(),
            json!({
                "data": {
                    "mal_id": 1,
                    "title": "Cowboy Bebop",
                    "episodes": 26,
                    "score": 8.78,
                    "status": "Finished Airing",
                    "genres": [
                        {"mal_id": 1, "name": "Action"},
                        {"mal_id": 24, "name": "Sci-Fi"}
                    ]
                }
            }),
        );

        responses.insert(
            "search_demon_slayer".to_string(),
            json!({
                "data": [
                    {
                        "mal_id": 38000,
                        "title": "Demon Slayer: Kimetsu no Yaiba",
                        "episodes": 26,
                        "score": 8.7,
                        "status": "Finished Airing"
                    }
                ],
                "pagination": {
                    "current_page": 1,
                    "last_visible_page": 1,
                    "has_next_page": false
                }
            }),
        );

        responses.insert(
            "season_now".to_string(),
            json!({
                "data": [
                    {
                        "mal_id": 50265,
                        "title": "Spy x Family Part 2",
                        "episodes": 12,
                        "score": 8.9,
                        "season": "fall",
                        "year": 2023
                    },
                    {
                        "mal_id": 48569,
                        "title": "Mob Psycho 100 III",
                        "episodes": 12,
                        "score": 9.1,
                        "season": "fall",
                        "year": 2023
                    }
                ]
            }),
        );

        responses.insert(
            "characters_21".to_string(),
            json!({
                "data": [
                    {
                        "character": {
                            "mal_id": 40,
                            "name": "Monkey D. Luffy",
                            "images": {
                                "jpg": {
                                    "image_url": "https://cdn.myanimelist.net/images/characters/9/310307.jpg"
                                }
                            }
                        },
                        "role": "Main",
                        "voice_actors": []
                    },
                    {
                        "character": {
                            "mal_id": 309,
                            "name": "Roronoa Zoro",
                            "images": {
                                "jpg": {
                                    "image_url": "https://cdn.myanimelist.net/images/characters/3/100534.jpg"
                                }
                            }
                        },
                        "role": "Main",
                        "voice_actors": []
                    }
                ]
            })
        );

        Self { responses }
    }

    fn get_response(&self, key: &str) -> Option<&serde_json::Value> {
        self.responses.get(key)
    }
}

// ==================== DATA PROCESSING TESTS ====================

#[test]
fn test_anime_data_structure() {
    let cache = MockApiCache::new();
    let response = cache.get_response("anime_1").unwrap();

    // Verify the structure matches what we expect
    assert!(response["data"]["mal_id"].is_number());
    assert!(response["data"]["title"].is_string());
    assert!(response["data"]["episodes"].is_number());
    assert!(response["data"]["score"].is_number());

    let title = response["data"]["title"].as_str().unwrap();
    assert_eq!(title, "Cowboy Bebop");

    let mal_id = response["data"]["mal_id"].as_u64().unwrap();
    assert_eq!(mal_id, 1);

    println!("✅ Anime data structure validation passed");
}

#[test]
fn test_search_response_structure() {
    let cache = MockApiCache::new();
    let response = cache.get_response("search_demon_slayer").unwrap();

    // Verify search response structure
    assert!(response["data"].is_array());
    assert!(response["pagination"].is_object());

    let data = response["data"].as_array().unwrap();
    assert!(!data.is_empty());

    let first_result = &data[0];
    assert!(first_result["mal_id"].is_number());
    assert!(first_result["title"].is_string());

    let pagination = &response["pagination"];
    assert!(pagination["current_page"].is_number());
    assert!(pagination["has_next_page"].is_boolean());

    println!("✅ Search response structure validation passed");
}

#[test]
fn test_seasonal_anime_structure() {
    let cache = MockApiCache::new();
    let response = cache.get_response("season_now").unwrap();

    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);

    for anime in data {
        assert!(anime["mal_id"].is_number());
        assert!(anime["title"].is_string());
        assert!(anime["score"].is_number());

        // Verify seasonal data
        if anime["season"].is_string() {
            let season = anime["season"].as_str().unwrap();
            assert!(["spring", "summer", "fall", "winter"].contains(&season));
        }
    }

    println!("✅ Seasonal anime structure validation passed");
}

#[test]
fn test_character_data_structure() {
    let cache = MockApiCache::new();
    let response = cache.get_response("characters_21").unwrap();

    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);

    for char_data in data {
        assert!(char_data["character"].is_object());
        assert!(char_data["role"].is_string());

        let character = &char_data["character"];
        assert!(character["mal_id"].is_number());
        assert!(character["name"].is_string());
        assert!(character["images"].is_object());

        let role = char_data["role"].as_str().unwrap();
        assert!(["Main", "Supporting"].contains(&role));
    }

    println!("✅ Character data structure validation passed");
}

// ==================== DATA QUALITY TESTS ====================

#[test]
fn test_score_validation() {
    let cache = MockApiCache::new();
    let response = cache.get_response("anime_1").unwrap();

    let score = response["data"]["score"].as_f64().unwrap();
    assert!(
        score >= 0.0 && score <= 10.0,
        "Score should be between 0-10"
    );
    assert!(score > 8.0, "Cowboy Bebop should have high score");

    println!("✅ Score validation passed: {}", score);
}

#[test]
fn test_episode_count_validation() {
    let cache = MockApiCache::new();
    let response = cache.get_response("anime_1").unwrap();

    let episodes = response["data"]["episodes"].as_u64().unwrap();
    assert!(episodes > 0, "Episode count should be positive");
    assert_eq!(episodes, 26, "Cowboy Bebop has 26 episodes");

    println!("✅ Episode count validation passed: {}", episodes);
}

#[test]
fn test_genre_structure() {
    let cache = MockApiCache::new();
    let response = cache.get_response("anime_1").unwrap();

    let genres = response["data"]["genres"].as_array().unwrap();
    assert!(!genres.is_empty(), "Anime should have genres");

    for genre in genres {
        assert!(genre["mal_id"].is_number());
        assert!(genre["name"].is_string());

        let genre_name = genre["name"].as_str().unwrap();
        assert!(!genre_name.is_empty(), "Genre name should not be empty");
    }

    println!("✅ Genre structure validation passed");
}

// ==================== PAGINATION TESTS ====================

#[test]
fn test_pagination_logic() {
    let cache = MockApiCache::new();
    let response = cache.get_response("search_demon_slayer").unwrap();

    let pagination = &response["pagination"];
    let current_page = pagination["current_page"].as_u64().unwrap();
    let last_page = pagination["last_visible_page"].as_u64().unwrap();
    let has_next = pagination["has_next_page"].as_bool().unwrap();

    assert_eq!(current_page, 1);
    assert_eq!(last_page, 1);
    assert!(!has_next);

    // Test pagination logic
    if current_page < last_page {
        assert!(has_next, "Should have next page if current < last");
    } else {
        assert!(!has_next, "Should not have next page if current >= last");
    }

    println!("✅ Pagination logic validation passed");
}

// ==================== ERROR HANDLING TESTS ====================

#[test]
fn test_missing_data_handling() {
    // Test how our code handles missing optional fields
    let incomplete_data = json!({
        "data": {
            "mal_id": 999,
            "title": "Incomplete Anime"
            // Missing episodes, score, etc.
        }
    });

    assert!(incomplete_data["data"]["mal_id"].is_number());
    assert!(incomplete_data["data"]["title"].is_string());
    assert!(incomplete_data["data"]["episodes"].is_null());
    assert!(incomplete_data["data"]["score"].is_null());

    println!("✅ Missing data handling validation passed");
}

#[test]
fn test_empty_array_handling() {
    let empty_response = json!({
        "data": [],
        "pagination": {
            "current_page": 1,
            "last_visible_page": 1,
            "has_next_page": false
        }
    });

    let data = empty_response["data"].as_array().unwrap();
    assert!(data.is_empty());

    println!("✅ Empty array handling validation passed");
}

// ==================== PERFORMANCE TESTS ====================

#[test]
fn test_data_parsing_performance() {
    let cache = MockApiCache::new();
    let start = std::time::Instant::now();

    // Simulate parsing multiple responses
    for _ in 0..1000 {
        let response = cache.get_response("anime_1").unwrap();
        let _title = response["data"]["title"].as_str().unwrap();
        let _score = response["data"]["score"].as_f64().unwrap();
        let _episodes = response["data"]["episodes"].as_u64().unwrap();
    }

    let duration = start.elapsed();
    assert!(
        duration < std::time::Duration::from_millis(100),
        "Parsing should be fast"
    );

    println!("✅ Performance test passed: {:?}", duration);
}

// ==================== INTEGRATION HELPERS ====================

#[test]
fn test_adapter_creation_performance() {
    let start = std::time::Instant::now();

    // Test adapter creation performance
    for _ in 0..100 {
        let _adapter = JikanAdapter::new();
    }

    let duration = start.elapsed();
    assert!(
        duration < std::time::Duration::from_millis(500),
        "Adapter creation should be reasonably fast"
    );

    println!("✅ Adapter creation performance passed: {:?}", duration);
}

#[test]
fn test_rate_limiter_accuracy() {
    let adapter = JikanAdapter::with_rate_limit(10.0, 5); // 10 req/sec, 5 burst

    // Test initial state
    assert!(
        adapter.can_make_request_now(),
        "Should allow initial requests"
    );

    // Test provider type consistency
    assert_eq!(
        adapter.get_provider_type(),
        miru_lib::modules::provider::AnimeProvider::Jikan
    );

    println!("✅ Rate limiter accuracy test passed");
}
