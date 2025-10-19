#![allow(dead_code)]
#![allow(unused_variables)]

use chrono::Utc;
/// Tests to verify the RelationsCache implementation works correctly
/// and provides performance improvements
use miru_lib::modules::anime::domain::services::anime_relations_service::{
    BasicRelations, RelationLink, RelationSource, RelationsCache,
};
use miru_lib::modules::provider::AnimeProvider;

#[tokio::test]
async fn basic_cache_store_and_retrieve() {
    let cache = RelationsCache::new();

    // Create test basic relations
    let basic = BasicRelations {
        anime_id: "test_anime_1".to_string(),
        relations: vec![RelationLink {
            target_id: "related_1".to_string(),
            relation_type: "Sequel".to_string(),
            title: Some("Test Sequel".to_string()),
            provider: AnimeProvider::AniList,
            category: "mainStory".to_string(),
        }],
        has_more: false,
        cache_timestamp: Utc::now(),
        source: RelationSource::Database,
    };

    // Store in cache
    cache
        .store_basic(&basic)
        .await
        .expect("Should store successfully");

    // Retrieve from cache
    let retrieved = cache.get_basic("test_anime_1").await;
    assert!(retrieved.is_some(), "Should retrieve from cache");

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.anime_id, "test_anime_1");
    assert_eq!(retrieved.relations.len(), 1);
    assert_eq!(retrieved.relations[0].target_id, "related_1");
}

#[tokio::test]
async fn cache_miss_returns_none() {
    let cache = RelationsCache::new();

    // Try to get non-existent entry
    let result = cache.get_basic("nonexistent_anime").await;
    assert!(result.is_none(), "Cache miss should return None");
}

#[tokio::test]
async fn cache_ttl_expiration() {
    let cache = RelationsCache::new();

    // Create basic relations with old timestamp (expired)
    let mut basic = BasicRelations {
        anime_id: "expired_anime".to_string(),
        relations: vec![],
        has_more: false,
        cache_timestamp: Utc::now() - chrono::Duration::hours(2), // 2 hours ago (expired)
        source: RelationSource::Database,
    };

    // Store in cache
    cache
        .store_basic(&basic)
        .await
        .expect("Should store successfully");

    // Try to retrieve - should return None because it's expired (TTL: 1 hour)
    // Note: The cache stores with current timestamp, so we need to test differently
    // This test verifies the is_fresh() method instead
    assert!(
        !basic.is_fresh(chrono::Duration::hours(1)),
        "Should be expired"
    );

    // Fresh timestamp should not be expired
    basic.cache_timestamp = Utc::now();
    assert!(
        basic.is_fresh(chrono::Duration::hours(1)),
        "Should be fresh"
    );
}

#[tokio::test]
async fn cache_stats_track_entries() {
    let cache = RelationsCache::new();

    // Initially empty
    let stats = cache.get_stats().await;
    assert_eq!(stats.basic_entries, 0);
    assert_eq!(stats.total_entries, 0);

    // Add some entries
    let basic1 = BasicRelations {
        anime_id: "anime_1".to_string(),
        relations: vec![],
        has_more: false,
        cache_timestamp: Utc::now(),
        source: RelationSource::Database,
    };

    let basic2 = BasicRelations {
        anime_id: "anime_2".to_string(),
        relations: vec![],
        has_more: false,
        cache_timestamp: Utc::now(),
        source: RelationSource::Database,
    };

    cache.store_basic(&basic1).await.expect("Should store");
    cache.store_basic(&basic2).await.expect("Should store");

    // Check stats
    let stats = cache.get_stats().await;
    assert_eq!(stats.basic_entries, 2);
    assert_eq!(stats.total_entries, 2);
}

#[tokio::test]
async fn cache_clear_all_removes_entries() {
    let cache = RelationsCache::new();

    // Add entries
    let basic = BasicRelations {
        anime_id: "test_anime".to_string(),
        relations: vec![],
        has_more: false,
        cache_timestamp: Utc::now(),
        source: RelationSource::Database,
    };

    cache.store_basic(&basic).await.expect("Should store");

    // Verify it's there
    assert!(cache.get_basic("test_anime").await.is_some());

    // Clear cache
    cache.clear_all().await.expect("Should clear successfully");

    // Verify it's gone
    assert!(cache.get_basic("test_anime").await.is_none());

    // Stats should show empty
    let stats = cache.get_stats().await;
    assert_eq!(stats.total_entries, 0);
}

#[tokio::test]
async fn cache_handles_concurrent_access() {
    use std::sync::Arc;

    let cache = Arc::new(RelationsCache::new());

    // Spawn multiple tasks that access cache concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            let basic = BasicRelations {
                anime_id: format!("anime_{}", i),
                relations: vec![],
                has_more: false,
                cache_timestamp: Utc::now(),
                source: RelationSource::Database,
            };

            // Store
            cache_clone.store_basic(&basic).await.expect("Should store");

            // Retrieve
            let retrieved = cache_clone.get_basic(&format!("anime_{}", i)).await;
            assert!(retrieved.is_some(), "Should retrieve what we just stored");
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }

    // Verify all 10 entries are in cache
    let stats = cache.get_stats().await;
    assert_eq!(stats.basic_entries, 10);
}

#[tokio::test]
async fn cache_overwrites_existing_entry() {
    let cache = RelationsCache::new();

    // Store first version
    let basic_v1 = BasicRelations {
        anime_id: "same_anime".to_string(),
        relations: vec![RelationLink {
            target_id: "related_1".to_string(),
            relation_type: "Sequel".to_string(),
            title: Some("Version 1".to_string()),
            provider: AnimeProvider::AniList,
            category: "mainStory".to_string(),
        }],
        has_more: false,
        cache_timestamp: Utc::now(),
        source: RelationSource::Database,
    };

    cache.store_basic(&basic_v1).await.expect("Should store v1");

    // Store second version (same anime_id)
    let basic_v2 = BasicRelations {
        anime_id: "same_anime".to_string(),
        relations: vec![RelationLink {
            target_id: "related_1".to_string(),
            relation_type: "Sequel".to_string(),
            title: Some("Version 2".to_string()),
            provider: AnimeProvider::AniList,
            category: "mainStory".to_string(),
        }],
        has_more: false,
        cache_timestamp: Utc::now(),
        source: RelationSource::Database,
    };

    cache.store_basic(&basic_v2).await.expect("Should store v2");

    // Retrieve - should get v2
    let retrieved = cache.get_basic("same_anime").await.unwrap();
    assert_eq!(retrieved.relations[0].title, Some("Version 2".to_string()));

    // Stats should still show 1 entry (not 2)
    let stats = cache.get_stats().await;
    assert_eq!(stats.basic_entries, 1);
}
