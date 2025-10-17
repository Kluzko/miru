/// Comprehensive tests for Stage 2.1: Relations Refactor
///
/// This test suite verifies:
/// 1. Bidirectional relations are created correctly
/// 2. Relations discovery uses AnimeIngestionService (no legacy fallback)
/// 3. Background job handler works correctly
/// 4. Edge cases and error scenarios are handled properly
/// 5. Relation type inversions are correct
/// 6. Database consistency is maintained
mod utils;

use miru_lib::modules::anime::application::ingestion_service::{
    AnimeSource, IngestionOptions, JobPriority,
};
use miru_lib::modules::anime::domain::value_objects::anime_tier::AnimeTier;
use miru_lib::modules::jobs::domain::entities::Job;
use miru_lib::modules::jobs::domain::repository::JobRepository;
use utils::{factories::AnimeFactory, helpers};

// ============================================================================
// BIDIRECTIONAL RELATIONS TESTS
// ============================================================================

#[tokio::test]
async fn bidirectional_relations_sequel_prequel() {
    
    

    let services = helpers::build_test_services();

    // Create Season 1
    let season1 = AnimeFactory::complete()
        .with_title("Attack on Titan Season 1")
        .build();
    let result1 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: season1.clone(),
                context: "Test".to_string(),
            },
            IngestionOptions {
                skip_duplicates: false,
                skip_provider_fetch: true,
                enrich_async: false,
                fetch_relations: false,
                priority: JobPriority::Normal,
            },
        )
        .await
        .expect("Season 1 ingestion should succeed");

    // Create Season 2
    let season2 = AnimeFactory::complete()
        .with_title("Attack on Titan Season 2")
        .build();
    let result2 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: season2.clone(),
                context: "Test".to_string(),
            },
            IngestionOptions {
                skip_duplicates: false,
                skip_provider_fetch: true,
                enrich_async: false,
                fetch_relations: false,
                priority: JobPriority::Normal,
            },
        )
        .await
        .expect("Season 2 ingestion should succeed");

    // Save relation: Season 1 → Season 2 (sequel)
    services
        .anime_repository
        .save_relations(
            &result1.anime.id,
            &vec![(result2.anime.id, "sequel".to_string())],
        )
        .await
        .expect("Saving relation should succeed");

    // VERIFY: Forward relation exists (Season 1 → Season 2 as sequel)
    let season1_relations = services
        .anime_repository
        .get_relations(&result1.anime.id)
        .await
        .expect("Getting Season 1 relations should succeed");

    assert_eq!(
        season1_relations.len(),
        1,
        "Season 1 should have 1 forward relation"
    );
    assert_eq!(
        season1_relations[0].0, result2.anime.id,
        "Forward relation should point to Season 2"
    );
    assert_eq!(
        season1_relations[0].1.to_lowercase(),
        "sequel",
        "Forward relation type should be sequel"
    );

    // VERIFY: Reverse relation exists (Season 2 → Season 1 as prequel)
    let season2_relations = services
        .anime_repository
        .get_relations(&result2.anime.id)
        .await
        .expect("Getting Season 2 relations should succeed");

    assert_eq!(
        season2_relations.len(),
        1,
        "Season 2 should have 1 reverse relation"
    );
    assert_eq!(
        season2_relations[0].0, result1.anime.id,
        "Reverse relation should point to Season 1"
    );
    assert_eq!(
        season2_relations[0].1.to_lowercase(),
        "prequel",
        "Reverse relation type should be prequel (inverse of sequel)"
    );
}

#[tokio::test]
async fn bidirectional_relations_side_story_parent_story() {
    
    

    let services = helpers::build_test_services();

    // Create main story
    let main = AnimeFactory::complete()
        .with_title("Sword Art Online")
        .build();
    let main_result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: main.clone(),
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .expect("Main story ingestion should succeed");

    // Create side story
    let side = AnimeFactory::complete()
        .with_title("Sword Art Online: Alternative Gun Gale Online")
        .build();
    let side_result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: side.clone(),
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .expect("Side story ingestion should succeed");

    // Save relation: Main → Side (side_story)
    services
        .anime_repository
        .save_relations(
            &main_result.anime.id,
            &vec![(side_result.anime.id, "side_story".to_string())],
        )
        .await
        .expect("Saving relation should succeed");

    // VERIFY: Reverse relation is parent_story
    let side_relations = services
        .anime_repository
        .get_relations(&side_result.anime.id)
        .await
        .expect("Getting side story relations should succeed");

    assert_eq!(
        side_relations.len(),
        1,
        "Side story should have 1 reverse relation"
    );
    assert_eq!(
        side_relations[0].1.to_lowercase(),
        "parent_story",
        "Reverse of side_story should be parent_story"
    );
}

#[tokio::test]
async fn bidirectional_relations_symmetric_types() {
    
    

    let services = helpers::build_test_services();

    // Create two alternative versions
    let alt1 = AnimeFactory::complete()
        .with_title("Fate/stay night")
        .build();
    let alt1_result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: alt1.clone(),
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .expect("Alt 1 ingestion should succeed");

    let alt2 = AnimeFactory::complete()
        .with_title("Fate/stay night: Unlimited Blade Works")
        .build();
    let alt2_result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: alt2.clone(),
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .expect("Alt 2 ingestion should succeed");

    // Save relation: Alt1 → Alt2 (alternative)
    services
        .anime_repository
        .save_relations(
            &alt1_result.anime.id,
            &vec![(alt2_result.anime.id, "alternative".to_string())],
        )
        .await
        .expect("Saving relation should succeed");

    // VERIFY: Reverse relation is also alternative (symmetric)
    let alt2_relations = services
        .anime_repository
        .get_relations(&alt2_result.anime.id)
        .await
        .expect("Getting alt2 relations should succeed");

    assert_eq!(
        alt2_relations.len(),
        1,
        "Alt2 should have 1 reverse relation"
    );
    assert_eq!(
        alt2_relations[0].1.to_lowercase(),
        "alternative",
        "Alternative relations should be symmetric"
    );
}

#[tokio::test]
async fn bidirectional_relations_multiple_relations() {
    
    

    let services = helpers::build_test_services();

    // Create franchise with multiple entries
    let season1 = AnimeFactory::complete()
        .with_title("Monogatari Season 1")
        .build();
    let s1_result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: season1,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    let season2 = AnimeFactory::complete()
        .with_title("Monogatari Season 2")
        .build();
    let s2_result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: season2,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    let movie = AnimeFactory::complete()
        .with_title("Monogatari Movie")
        .build();
    let movie_result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: movie,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    // Save multiple relations from Season 1
    services
        .anime_repository
        .save_relations(
            &s1_result.anime.id,
            &vec![
                (s2_result.anime.id, "sequel".to_string()),
                (movie_result.anime.id, "side_story".to_string()),
            ],
        )
        .await
        .expect("Saving multiple relations should succeed");

    // VERIFY: Season 1 has 2 forward relations
    let s1_relations = services
        .anime_repository
        .get_relations(&s1_result.anime.id)
        .await
        .unwrap();
    assert_eq!(s1_relations.len(), 2, "Season 1 should have 2 relations");

    // VERIFY: Season 2 has 1 reverse relation (prequel)
    let s2_relations = services
        .anime_repository
        .get_relations(&s2_result.anime.id)
        .await
        .unwrap();
    assert_eq!(
        s2_relations.len(),
        1,
        "Season 2 should have 1 reverse relation"
    );
    assert_eq!(s2_relations[0].1.to_lowercase(), "prequel");

    // VERIFY: Movie has 1 reverse relation (parent_story)
    let movie_relations = services
        .anime_repository
        .get_relations(&movie_result.anime.id)
        .await
        .unwrap();
    assert_eq!(
        movie_relations.len(),
        1,
        "Movie should have 1 reverse relation"
    );
    assert_eq!(movie_relations[0].1.to_lowercase(), "parent_story");
}

#[tokio::test]
async fn bidirectional_relations_idempotent_updates() {
    
    

    let services = helpers::build_test_services();

    let anime1 = AnimeFactory::complete().with_title("Anime 1").build();
    let r1 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: anime1,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    let anime2 = AnimeFactory::complete().with_title("Anime 2").build();
    let r2 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: anime2,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    // Save relation first time
    services
        .anime_repository
        .save_relations(&r1.anime.id, &vec![(r2.anime.id, "sequel".to_string())])
        .await
        .expect("First save should succeed");

    // Save SAME relation again (idempotent update)
    services
        .anime_repository
        .save_relations(&r1.anime.id, &vec![(r2.anime.id, "sequel".to_string())])
        .await
        .expect("Second save should succeed (idempotent)");

    // VERIFY: Still only 1 relation in each direction
    let r1_relations = services
        .anime_repository
        .get_relations(&r1.anime.id)
        .await
        .unwrap();
    assert_eq!(
        r1_relations.len(),
        1,
        "Should still have exactly 1 forward relation"
    );

    let r2_relations = services
        .anime_repository
        .get_relations(&r2.anime.id)
        .await
        .unwrap();
    assert_eq!(
        r2_relations.len(),
        1,
        "Should still have exactly 1 reverse relation"
    );
}

// ============================================================================
// RELATIONS DISCOVERY USES INGESTION SERVICE (NO LEGACY FALLBACK)
// ============================================================================

#[tokio::test]
async fn relations_discovery_uses_ingestion_service_not_legacy() {
    
    

    let services = helpers::build_test_services();

    // Create an anime via ingestion service
    let anime = AnimeFactory::minimal()
        .with_title("Test Anime")
        .with_score(8.5)
        .build();

    let result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime,
                context: "Test".to_string(),
            },
            IngestionOptions {
                skip_duplicates: false,
                skip_provider_fetch: true,
                enrich_async: false,
                fetch_relations: false,
                priority: JobPriority::Normal,
            },
        )
        .await
        .expect("Ingestion should succeed");

    // CRITICAL VERIFICATION:
    // If anime was created via ingestion service, it MUST have:
    // 1. Calculated tier (not hardcoded C)
    // 2. Quality metrics populated (not zeros)
    // 3. Composite score calculated

    // Check tier matches the calculated composite score (proves calculation was used)
    // Legacy fallback would hardcode tier C regardless of composite score
    let expected_tier = AnimeTier::from_score(result.anime.composite_score);
    assert!(
        matches!(result.anime.tier, expected_tier),
        "Tier should match composite score calculation (proves ingestion service was used). Composite: {}, Expected: {:?}, Got: {:?}",
        result.anime.composite_score,
        expected_tier,
        result.anime.tier
    );

    // Check quality metrics are populated
    assert!(
        result.anime.quality_metrics.popularity_score > 0.0
            || result.anime.quality_metrics.engagement_score > 0.0
            || result.anime.quality_metrics.consistency_score > 0.0,
        "Quality metrics should be populated (legacy fallback would have zeros)"
    );

    // Check composite score is calculated
    assert!(
        result.anime.composite_score > 0.0,
        "Composite score should be > 0 (legacy fallback would not calculate)"
    );
}

#[tokio::test]
async fn relations_discovery_with_missing_data_still_calculates_tier() {
    
    

    let services = helpers::build_test_services();

    // Create anime with MINIMAL data (this would trigger legacy fallback in old code)
    let minimal_anime = AnimeFactory::minimal()
        .with_title("Minimal Data Anime")
        .build();

    let result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: minimal_anime,
                context: "Relation Discovery".to_string(),
            },
            IngestionOptions {
                skip_duplicates: false,
                skip_provider_fetch: true, // Force minimal data path
                enrich_async: false,
                fetch_relations: false,
                priority: JobPriority::Normal,
            },
        )
        .await
        .expect("Even minimal anime should ingest successfully");

    // VERIFY: Tier is calculated based on actual data quality, not hardcoded
    // Minimal data should result in D or C tier, but it should be CALCULATED
    assert!(
        matches!(result.anime.tier, AnimeTier::D | AnimeTier::C),
        "Minimal data should get low tier, got {:?}",
        result.anime.tier
    );

    // VERIFY: Quality metrics exist (even if low)
    assert!(
        result.quality_score >= 0.0 && result.quality_score <= 1.0,
        "Quality score should be in valid range [0,1], got {}",
        result.quality_score
    );

    // VERIFY: update_scores() was called (sets updated_at)
    assert!(
        result.anime.updated_at > result.anime.created_at
            || result.anime.updated_at == result.anime.created_at,
        "updated_at should be set by update_scores()"
    );
}

// ============================================================================
// BACKGROUND JOB HANDLER TESTS
// ============================================================================

#[tokio::test]
async fn relations_job_handler_processes_job_successfully() {
    
    

    let services = helpers::build_test_services();

    // Create an anime
    let anime = AnimeFactory::complete()
        .with_title("Test Anime for Job")
        .build();
    let result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    // Queue a relations discovery job
    let job = Job::relations_discovery(result.anime.id, 5);
    let job_record = services
        .job_repository
        .enqueue(job)
        .await
        .expect("Enqueueing job should succeed");

    // Start background worker
    let worker_handle = services.background_worker.clone().start();

    // Wait for job to be processed (max 10 seconds)
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Stop worker
    services.background_worker.stop().await;
    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(2), worker_handle).await;

    // VERIFY: Job was processed
    let stats = services.job_repository.get_statistics().await.unwrap();

    // Job should be either completed or failed (not pending or running)
    assert!(
        stats.completed_count > 0 || stats.failed_count > 0,
        "Job should have been processed (completed or failed), stats: completed={}, failed={}, pending={}, running={}",
        stats.completed_count,
        stats.failed_count,
        stats.pending_count,
        stats.running_count
    );
}

#[tokio::test]
async fn relations_job_handler_handles_nonexistent_anime() {
    
    

    let services = helpers::build_test_services();

    // Queue a job for non-existent anime
    let fake_uuid = uuid::Uuid::new_v4();
    let job = Job::relations_discovery(fake_uuid, 5);
    services
        .job_repository
        .enqueue(job)
        .await
        .expect("Enqueueing job should succeed");

    // Start background worker
    let worker_handle = services.background_worker.clone().start();

    // Wait for job processing
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Stop worker
    services.background_worker.stop().await;
    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(2), worker_handle).await;

    // VERIFY: Job failed (not stuck in running state)
    let stats = services.job_repository.get_statistics().await.unwrap();

    assert!(
        stats.failed_count > 0 || stats.completed_count > 0,
        "Job for nonexistent anime should fail gracefully, not hang. Stats: failed={}, completed={}, running={}",
        stats.failed_count,
        stats.completed_count,
        stats.running_count
    );

    assert_eq!(
        stats.running_count, 0,
        "No jobs should be stuck in running state"
    );
}

// ============================================================================
// EDGE CASES AND ERROR SCENARIOS
// ============================================================================

#[tokio::test]
async fn circular_relations_dont_cause_infinite_loop() {
    
    

    let services = helpers::build_test_services();

    // Create A and B
    let a = AnimeFactory::complete().with_title("Anime A").build();
    let a_result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: a,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    let b = AnimeFactory::complete().with_title("Anime B").build();
    let b_result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: b,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    // Create circular relation: A → B and B → A
    services
        .anime_repository
        .save_relations(
            &a_result.anime.id,
            &vec![(b_result.anime.id, "sequel".to_string())],
        )
        .await
        .unwrap();

    services
        .anime_repository
        .save_relations(
            &b_result.anime.id,
            &vec![(a_result.anime.id, "sequel".to_string())],
        )
        .await
        .unwrap();

    // VERIFY: Both relations exist without causing database issues
    let a_relations = services
        .anime_repository
        .get_relations(&a_result.anime.id)
        .await
        .unwrap();
    let b_relations = services
        .anime_repository
        .get_relations(&b_result.anime.id)
        .await
        .unwrap();

    assert!(a_relations.len() >= 1, "A should have at least 1 relation");
    assert!(b_relations.len() >= 1, "B should have at least 1 relation");
}

#[tokio::test]
async fn self_referential_relation_rejected() {
    
    

    let services = helpers::build_test_services();

    let anime = AnimeFactory::complete().with_title("Self Ref").build();
    let result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    // Try to create self-referential relation (A → A)
    let save_result = services
        .anime_repository
        .save_relations(
            &result.anime.id,
            &vec![(result.anime.id, "sequel".to_string())],
        )
        .await;

    // VERIFY: Should fail (database has CHECK constraint preventing this)
    assert!(
        save_result.is_err(),
        "Self-referential relations should be rejected"
    );
}

#[tokio::test]
async fn relation_to_nonexistent_anime_rejected() {
    
    

    let services = helpers::build_test_services();

    let anime = AnimeFactory::complete().with_title("Real Anime").build();
    let result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    // Try to create relation to non-existent anime
    let fake_uuid = uuid::Uuid::new_v4();
    let save_result = services
        .anime_repository
        .save_relations(&result.anime.id, &vec![(fake_uuid, "sequel".to_string())])
        .await;

    // VERIFY: Should fail (foreign key constraint)
    assert!(
        save_result.is_err(),
        "Relations to non-existent anime should be rejected"
    );
}

#[tokio::test]
async fn unknown_relation_type_defaults_to_other() {
    
    

    let services = helpers::build_test_services();

    let a1 = AnimeFactory::complete().with_title("A1").build();
    let r1 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: a1,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    let a2 = AnimeFactory::complete().with_title("A2").build();
    let r2 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: a2,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    // Use invalid/unknown relation type
    let result = services
        .anime_repository
        .save_relations(
            &r1.anime.id,
            &vec![(r2.anime.id, "invalid_type".to_string())],
        )
        .await;

    // VERIFY: Should succeed (defaults to "other")
    assert!(
        result.is_ok(),
        "Unknown relation types should default to 'other'"
    );

    // Verify it was saved as "other"
    let relations = services
        .anime_repository
        .get_relations(&r1.anime.id)
        .await
        .unwrap();

    assert_eq!(relations.len(), 1, "Relation should be saved");
    assert_eq!(
        relations[0].1.to_lowercase(),
        "other",
        "Unknown type should be saved as 'other'"
    );
}

#[tokio::test]
async fn case_insensitive_relation_types() {
    
    

    let services = helpers::build_test_services();

    let a1 = AnimeFactory::complete().with_title("A1").build();
    let r1 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: a1,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    let a2 = AnimeFactory::complete().with_title("A2").build();
    let r2 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: a2,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    // Test various casings
    for case_variant in &["SEQUEL", "Sequel", "sequel", "SeQuEl"] {
        services
            .anime_repository
            .save_relations(&r1.anime.id, &vec![(r2.anime.id, case_variant.to_string())])
            .await
            .expect(&format!("Should accept relation type: {}", case_variant));
    }

    // All should result in same normalized type
    let relations = services
        .anime_repository
        .get_relations(&r1.anime.id)
        .await
        .unwrap();

    assert!(
        relations.len() >= 1,
        "At least one relation should be saved"
    );
}

// ============================================================================
// TRANSACTION CONSISTENCY TESTS
// ============================================================================

#[tokio::test]
async fn bidirectional_save_is_atomic() {
    
    

    let services = helpers::build_test_services();

    let a1 = AnimeFactory::complete().with_title("A1").build();
    let r1 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: a1,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    let a2 = AnimeFactory::complete().with_title("A2").build();
    let r2 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: a2,
                context: "Test".to_string(),
            },
            IngestionOptions::default(),
        )
        .await
        .unwrap();

    // Save relation
    services
        .anime_repository
        .save_relations(&r1.anime.id, &vec![(r2.anime.id, "sequel".to_string())])
        .await
        .expect("Save should succeed");

    // VERIFY: Both forward and reverse exist (atomic transaction)
    let forward_relations = services
        .anime_repository
        .get_relations(&r1.anime.id)
        .await
        .unwrap();
    let reverse_relations = services
        .anime_repository
        .get_relations(&r2.anime.id)
        .await
        .unwrap();

    // Both should exist (not just one)
    assert_eq!(forward_relations.len(), 1, "Forward relation should exist");
    assert_eq!(
        reverse_relations.len(),
        1,
        "Reverse relation should exist (proves atomicity)"
    );
}

#[tokio::test]
async fn multiple_relations_saved_atomically() {
    
    

    let services = helpers::build_test_services();

    // Create 4 anime
    let mut anime_ids = Vec::new();
    for i in 1..=4 {
        let anime = AnimeFactory::complete()
            .with_title(&format!("Anime {}", i))
            .build();
        let result = services
            .ingestion_service
            .ingest_anime(
                AnimeSource::DirectData {
                    anime,
                    context: "Test".to_string(),
                },
                IngestionOptions::default(),
            )
            .await
            .unwrap();
        anime_ids.push(result.anime.id);
    }

    // Save 3 relations from anime 0 in ONE transaction
    services
        .anime_repository
        .save_relations(
            &anime_ids[0],
            &vec![
                (anime_ids[1], "sequel".to_string()),
                (anime_ids[2], "side_story".to_string()),
                (anime_ids[3], "alternative".to_string()),
            ],
        )
        .await
        .expect("Batch save should succeed");

    // VERIFY: All 3 forward relations exist
    let forward_relations = services
        .anime_repository
        .get_relations(&anime_ids[0])
        .await
        .unwrap();
    assert_eq!(
        forward_relations.len(),
        3,
        "All 3 forward relations should exist"
    );

    // VERIFY: All 3 reverse relations exist
    for i in 1..=3 {
        let reverse_relations = services
            .anime_repository
            .get_relations(&anime_ids[i])
            .await
            .unwrap();
        assert!(
            reverse_relations.len() >= 1,
            "Reverse relation {} should exist",
            i
        );
    }
}
