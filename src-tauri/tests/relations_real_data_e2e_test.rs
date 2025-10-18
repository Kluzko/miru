/// These tests use REAL anime data from AniList to verify:
/// 1. Relations discovery fetches and saves real franchise relationships
/// 2. Bidirectional relations are created correctly
/// 3. No legacy fallback is used (proper tier calculation)
/// 4. Background jobs process real relations correctly
///
/// ⚠️ WARNING: These tests:
/// - Make real HTTP requests to AniList API
/// - May be rate-limited or fail if API is down
/// - Take longer to run (30-60 seconds each)
/// - Should be run separately from unit tests
/// - Each test creates an isolated database that is automatically cleaned up
mod utils;

use futures::future::BoxFuture;
use miru_lib::modules::anime::{
    application::ingestion_service::{AnimeSource, IngestionOptions},
    domain::value_objects::anime_tier::AnimeTier,
    JobPriority,
};
use std::time::Duration;
use tokio::time::sleep;
use utils::{helpers, test_db::TestDb};

/// Test: Import a real anime with a sequel and verify bidirectional relations
///
/// Uses: Death Note (AniList ID: 1535) which has Death Note Relight as a special
#[tokio::test]
#[ignore] // Run with: cargo test --test stage_2_1_real_data_e2e_test -- --ignored
async fn e2e_real_anime_has_bidirectional_relations() {
    let test_db = TestDb::new();

    test_db
        .run_test(|pool| {
            Box::pin(async move {
                let services = helpers::build_test_services_with_pool(pool);

                println!("\n=== Testing Real Anime Bidirectional Relations ===");
                println!("Importing Death Note from AniList...\n");

                // Step 1: Import Death Note via manual search (will fetch from AniList)
                let death_note_result = services
                    .ingestion_service
                    .ingest_anime(
                        AnimeSource::ManualImport {
                            title: "Death Note".to_string(),
                        },
                        IngestionOptions {
                            skip_duplicates: false,
                            skip_provider_fetch: false, // Fetch from AniList
                            enrich_async: false,
                            fetch_relations: true, // Queue relations discovery job
                            priority: JobPriority::High,
                        },
                    )
                    .await
                    .expect("Should import Death Note from AniList");

                println!("✓ Imported: {}", death_note_result.anime.title.main);
                println!("  - ID: {}", death_note_result.anime.id);
                println!("  - Score: {:?}", death_note_result.anime.score);
                println!("  - Tier: {:?}", death_note_result.anime.tier);
                println!("  - Composite: {}", death_note_result.anime.composite_score);

                // Verify proper tier calculation (not legacy fallback)
                assert!(
                    death_note_result.anime.composite_score > 0.0,
                    "Should have calculated composite score from real data"
                );
                assert!(
                    death_note_result.anime.score.unwrap_or(0.0) > 8.0,
                    "Death Note should have high score from AniList"
                );
                assert!(
                    matches!(death_note_result.anime.tier, AnimeTier::S | AnimeTier::A),
                    "Death Note should get S or A tier, not C (legacy), got: {:?}",
                    death_note_result.anime.tier
                );

                // Step 2: Wait for relations discovery job to complete
                println!("\nWaiting for relations discovery job to process...");

                // Start background worker
                let worker = services.background_worker.clone();
                let worker_handle = tokio::spawn(async move { worker.run().await });

                // Wait for job to complete (check every 2 seconds for up to 30 seconds)
                let mut relations_found = false;
                for attempt in 1..=15 {
                    sleep(Duration::from_secs(2)).await;

                    let relations = services
                        .anime_repository
                        .get_relations(&death_note_result.anime.id)
                        .await
                        .expect("Should get relations");

                    if !relations.is_empty() {
                        println!("✓ Relations discovered after {} seconds", attempt * 2);
                        println!("  Found {} related anime", relations.len());
                        relations_found = true;
                        break;
                    }
                }

                // Stop worker
                services.background_worker.stop().await;
                let _ = worker_handle.await;

                assert!(
                    relations_found,
                    "Death Note should have related anime (specials, movies, etc.)"
                );

                // Step 3: Verify bidirectional relations
                println!("\nVerifying bidirectional relations...");

                let death_note_relations = services
                    .anime_repository
                    .get_relations(&death_note_result.anime.id)
                    .await
                    .expect("Should get relations");

                assert!(
                    !death_note_relations.is_empty(),
                    "Death Note should have related anime"
                );

                // Check each forward relation has a corresponding reverse relation
                for (related_id, forward_type) in &death_note_relations {
                    println!("  Checking relation: {} ({})", related_id, forward_type);

                    let reverse_relations = services
                        .anime_repository
                        .get_relations(related_id)
                        .await
                        .expect("Should get reverse relations");

                    let has_bidirectional = reverse_relations
                        .iter()
                        .any(|(id, _)| *id == death_note_result.anime.id);

                    assert!(
                        has_bidirectional,
                        "Related anime {} should have bidirectional relation back to Death Note",
                        related_id
                    );
                }

                println!(
                    "✓ All {} relations are bidirectional",
                    death_note_relations.len()
                );
                println!("\n=== Test Complete ===\n");
            }) as BoxFuture<'static, ()>
        })
        .await;
} // Database automatically cleaned up here

/// Test: Import high-score anime and verify proper tier calculation (not legacy C)
///
/// Uses: Fullmetal Alchemist: Brotherhood (one of highest-rated anime)
#[tokio::test]
#[ignore]
async fn e2e_high_score_anime_gets_proper_tier_not_legacy() {
    let test_db = TestDb::new();

    test_db
        .run_test(|pool| {
            Box::pin(async move {
                let services = helpers::build_test_services_with_pool(pool);

                println!("\n=== Testing High-Score Anime Tier Calculation ===");
                println!("Importing Fullmetal Alchemist: Brotherhood...\n");

                let result = services
                    .ingestion_service
                    .ingest_anime(
                        AnimeSource::ManualImport {
                            title: "Fullmetal Alchemist Brotherhood".to_string(),
                        },
                        IngestionOptions {
                            skip_duplicates: false,
                            skip_provider_fetch: false,
                            enrich_async: false,
                            fetch_relations: false,
                            priority: JobPriority::Normal,
                        },
                    )
                    .await
                    .expect("Should import FMAB from AniList");

                println!("✓ Imported: {}", result.anime.title.main);
                println!("  - Score: {:?}", result.anime.score);
                println!("  - Composite: {}", result.anime.composite_score);
                println!("  - Tier: {:?}", result.anime.tier);
                println!("  - Age Restriction: {:?}", result.anime.age_restriction);
                println!(
                    "  - Popularity: {}",
                    result.anime.quality_metrics.popularity_score
                );
                println!(
                    "  - Engagement: {}",
                    result.anime.quality_metrics.engagement_score
                );

                // Verify composite score was calculated (not 0.0 from legacy)
                assert!(
                    result.anime.composite_score > 0.0,
                    "Composite score should be calculated, got: {}",
                    result.anime.composite_score
                );

                // Verify score is high
                let score = result.anime.score.expect("FMAB should have a score");
                assert!(
                    score >= 8.5,
                    "FMAB should have very high score, got: {}",
                    score
                );

                // CRITICAL: Verify tier is S or A, NOT C (which would indicate legacy fallback)
                assert!(
                    matches!(result.anime.tier, AnimeTier::S | AnimeTier::A),
                    "FMAB with score {} should get S or A tier, NOT C (legacy fallback). Got: {:?}",
                    score,
                    result.anime.tier
                );

                // Verify tier matches composite score (proves calculation was used)
                let expected_tier = AnimeTier::from_score(result.anime.composite_score);
                assert_eq!(
                    result.anime.tier, expected_tier,
                    "Tier should match composite score calculation"
                );

                // Verify quality metrics are populated (not zeros from legacy)
                assert!(
                    result.anime.quality_metrics.popularity_score > 0.0,
                    "Popularity score should be calculated"
                );
                assert!(
                    result.anime.quality_metrics.engagement_score > 0.0
                        || result.anime.quality_metrics.consistency_score > 0.0,
                    "At least one quality metric should be populated"
                );

                // CRITICAL: Verify age_restriction is populated
                // AniList doesn't provide age_restriction, so it must come from enrichment (Jikan)
                // This proves the enrichment service is working correctly
                assert!(
                    result.anime.age_restriction.is_some(),
                    "Age restriction should be populated by enrichment service (Jikan), got None. \
                     AniList doesn't provide this field, so enrichment MUST fetch it from Jikan."
                );

                println!(
                    "✓ Age restriction populated: {:?} (enrichment service worked)",
                    result.anime.age_restriction.unwrap()
                );

                println!("\n✓ High-score anime has proper tier calculation (not legacy fallback)");
                println!("✓ Enrichment service is working (age_restriction populated from Jikan)");
                println!("=== Test Complete ===\n");
            }) as BoxFuture<'static, ()>
        })
        .await;
} // Database automatically cleaned up here

/// Test: Import anime with complex franchise relations
///
/// Uses: Attack on Titan (has multiple seasons, OVAs, movies)
#[tokio::test]
#[ignore]
async fn e2e_complex_franchise_all_relations_bidirectional() {
    let test_db = TestDb::new();

    test_db
        .run_test(|pool| {
            Box::pin(async move {
                let services = helpers::build_test_services_with_pool(pool);

                println!("\n=== Testing Complex Franchise Relations ===");
                println!("Importing Attack on Titan...\n");

                // Import Season 1
                let season1_result = services
                    .ingestion_service
                    .ingest_anime(
                        AnimeSource::ManualImport {
                            title: "Shingeki no Kyojin".to_string(),
                        },
                        IngestionOptions {
                            skip_duplicates: false,
                            skip_provider_fetch: false,
                            enrich_async: false,
                            fetch_relations: true, // Discover entire franchise
                            priority: JobPriority::High,
                        },
                    )
                    .await
                    .expect("Should import Attack on Titan");

                println!("✓ Imported: {}", season1_result.anime.title.main);
                println!("  - ID: {}", season1_result.anime.id);

                // Start background worker to process relations job
                println!("\nStarting background worker to discover franchise...");
                let worker = services.background_worker.clone();
                let worker_handle = tokio::spawn(async move { worker.run().await });

                // Wait for relations to be discovered (up to 45 seconds)
                let mut relations_count = 0;
                for attempt in 1..=15 {
                    sleep(Duration::from_secs(3)).await;

                    let relations = services
                        .anime_repository
                        .get_relations(&season1_result.anime.id)
                        .await
                        .expect("Should get relations");

                    if !relations.is_empty() {
                        relations_count = relations.len();
                        println!(
                            "✓ Found {} relations after {} seconds",
                            relations_count,
                            attempt * 3
                        );
                        break;
                    }
                }

                services.background_worker.stop().await;
                let _ = worker_handle.await;

                assert!(
                    relations_count > 0,
                    "Attack on Titan should have multiple related anime (seasons, OVAs, movies)"
                );

                // Get all relations
                let all_relations = services
                    .anime_repository
                    .get_relations(&season1_result.anime.id)
                    .await
                    .expect("Should get relations");

                println!("\nRelations found:");
                for (related_id, rel_type) in &all_relations {
                    println!("  - {} ({})", related_id, rel_type);
                }

                // CRITICAL: Verify EVERY relation is bidirectional
                println!("\nVerifying all relations are bidirectional...");
                let mut bidirectional_count = 0;
                for (related_id, forward_type) in &all_relations {
                    let reverse_relations = services
                        .anime_repository
                        .get_relations(related_id)
                        .await
                        .expect("Should get reverse relations");

                    let has_reverse = reverse_relations
                        .iter()
                        .any(|(id, _)| *id == season1_result.anime.id);

                    assert!(
                        has_reverse,
                        "Relation to {} ({}) should be bidirectional",
                        related_id, forward_type
                    );
                    bidirectional_count += 1;
                }

                println!("✓ All {} relations are bidirectional", bidirectional_count);

                // Verify all discovered anime have proper tier calculation (not legacy)
                println!("\nVerifying all discovered anime used ingestion service...");
                let mut checked_anime = 0;
                for (related_id, _) in &all_relations {
                    if let Ok(Some(related_anime)) =
                        services.anime_service.get_anime_by_id(related_id).await
                    {
                        assert!(
                            related_anime.composite_score > 0.0,
                            "Anime '{}' should have calculated composite score (not legacy)",
                            related_anime.title.main
                        );
                        checked_anime += 1;
                    }
                }

                println!(
                    "✓ Checked {} related anime - all have proper calculations",
                    checked_anime
                );

                println!("\n=== Test Complete ===\n");
            }) as BoxFuture<'static, ()>
        })
        .await;
} // Database automatically cleaned up here

/// Test: Verify idempotent relations discovery (no duplicates on re-discovery)
#[tokio::test]
#[ignore]
async fn e2e_relations_discovery_is_idempotent() {
    let test_db = TestDb::new();

    test_db
        .run_test(|pool| {
            Box::pin(async move {
                let services = helpers::build_test_services_with_pool(pool);

                println!("\n=== Testing Idempotent Relations Discovery ===");

                // Import anime
                let result = services
                    .ingestion_service
                    .ingest_anime(
                        AnimeSource::ManualImport {
                            title: "Steins Gate".to_string(),
                        },
                        IngestionOptions {
                            skip_duplicates: false,
                            skip_provider_fetch: false,
                            enrich_async: false,
                            fetch_relations: true,
                            priority: JobPriority::High,
                        },
                    )
                    .await
                    .expect("Should import Steins;Gate");

                println!("✓ Imported: {}", result.anime.title.main);

                // Process first relations discovery
                let worker = services.background_worker.clone();
                let worker_handle = tokio::spawn(async move { worker.run().await });
                sleep(Duration::from_secs(15)).await;
                services.background_worker.stop().await;
                let _ = worker_handle.await;

                // Get initial relations count
                let initial_relations = services
                    .anime_repository
                    .get_relations(&result.anime.id)
                    .await
                    .expect("Should get relations");
                let initial_count = initial_relations.len();

                println!("\nInitial relations count: {}", initial_count);

                // Re-import the same anime with relations discovery again
                println!("\nRe-discovering relations...");
                let _result2 = services
                    .ingestion_service
                    .ingest_anime(
                        AnimeSource::ManualImport {
                            title: "Steins Gate".to_string(),
                        },
                        IngestionOptions {
                            skip_duplicates: false, // Allow re-import
                            skip_provider_fetch: false,
                            enrich_async: false,
                            fetch_relations: true, // Discover relations again
                            priority: JobPriority::High,
                        },
                    )
                    .await
                    .expect("Should re-import Steins;Gate");

                // Process second relations discovery
                let worker2 = services.background_worker.clone();
                let worker_handle2 = tokio::spawn(async move { worker2.run().await });
                sleep(Duration::from_secs(15)).await;
                services.background_worker.stop().await;
                let _ = worker_handle2.await;

                // Get final relations count
                let final_relations = services
                    .anime_repository
                    .get_relations(&result.anime.id)
                    .await
                    .expect("Should get relations");
                let final_count = final_relations.len();

                println!("Final relations count: {}", final_count);

                // CRITICAL: Count should be the same (no duplicates)
                assert_eq!(
                    initial_count, final_count,
                    "Relations count should not change on re-discovery (idempotent). Initial: {}, Final: {}",
                    initial_count, final_count
                );

                println!("\n✓ Relations discovery is idempotent (no duplicates created)");
                println!("=== Test Complete ===\n");
            }) as BoxFuture<'static, ()>
        })
        .await;
} // Database automatically cleaned up here
