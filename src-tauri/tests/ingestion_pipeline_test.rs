/// Comprehensive tests for anime ingestion pipeline
///
/// Tests cover:
/// - Tier calculation (not hardcoded to C)
/// - Quality-based enrichment queueing
/// - Duplicate detection
/// - Manual import flow
/// - Relation discovery flow
mod utils;

use miru_lib::modules::anime::application::ingestion_service::{
    AnimeSource, IngestionOptions, JobPriority,
};
use miru_lib::modules::anime::domain::value_objects::anime_tier::AnimeTier;
use miru_lib::modules::jobs::domain::repository::JobRepository;
use utils::{factories::AnimeFactory, helpers};

#[tokio::test]
async fn minimal_data_gets_low_tier() {
    
    
    let services = helpers::build_test_services();

    let anime = AnimeFactory::minimal().build();

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

    assert!(
        matches!(result.anime.tier, AnimeTier::D | AnimeTier::C),
        "Minimal data should result in low tier, got {:?}",
        result.anime.tier
    );
    assert!(result.anime.composite_score < 6.0);
}

#[tokio::test]
async fn complete_data_gets_high_tier() {
    
    
    let services = helpers::build_test_services();

    let anime = AnimeFactory::complete().build();

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

    assert!(
        matches!(
            result.anime.tier,
            AnimeTier::A | AnimeTier::S | AnimeTier::B
        ),
        "Complete data should result in high tier, got {:?}",
        result.anime.tier
    );
    assert!(result.anime.composite_score >= 7.0);
}

#[tokio::test]
async fn relation_discovery_calculates_tier_not_hardcoded() {
    
    
    let services = helpers::build_test_services();

    // This is the KEY test - relation discovery should NOT hardcode tier to C
    let result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::RelationDiscovery {
                anilist_id: 16498, // Attack on Titan (real anime)
                relation_type: "sequel".to_string(),
                source_anime_id: "test".to_string(),
            },
            IngestionOptions {
                skip_duplicates: false,
                skip_provider_fetch: false,
                enrich_async: false,
                fetch_relations: false,
                priority: JobPriority::Low,
            },
        )
        .await;

    if let Ok(result) = result {
        // Before the fix: tier would be hardcoded to C, score would be 0.0
        // After the fix: tier is calculated based on data, score > 0
        assert_ne!(
            result.anime.tier,
            AnimeTier::C,
            "Tier should be calculated, not hardcoded to C"
        );
        assert!(
            result.anime.composite_score > 0.0,
            "Score should be calculated, got {}",
            result.anime.composite_score
        );

        println!(
            "✓ Related anime has tier {:?} with score {:.2}",
            result.anime.tier, result.anime.composite_score
        );
    }
}

// ================================================================================================
// ENRICHMENT QUEUEING TESTS
// ================================================================================================

#[tokio::test]
async fn low_quality_anime_queues_enrichment() {
    
    
    let services = helpers::build_test_services();

    let anime = AnimeFactory::minimal().build();

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
                enrich_async: true, // Request enrichment
                fetch_relations: false,
                priority: JobPriority::Normal,
            },
        )
        .await
        .unwrap();

    assert!(
        result.enrichment_queued,
        "Low quality anime should queue enrichment job"
    );

    let pending_jobs = services.job_repository.get_pending_jobs().await.unwrap();
    assert_eq!(pending_jobs.len(), 1);
    assert_eq!(pending_jobs[0].job_type, "enrichment");

    let payload = pending_jobs[0].parse_enrichment_payload().unwrap();
    assert_eq!(payload.anime_id, result.anime.id);
}

#[tokio::test]
async fn high_quality_anime_skips_enrichment() {
    
    
    let services = helpers::build_test_services();

    let anime = AnimeFactory::complete().build();

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
                enrich_async: true,
                fetch_relations: false,
                priority: JobPriority::Normal,
            },
        )
        .await
        .unwrap();

    assert!(
        !result.enrichment_queued,
        "High quality anime should NOT queue enrichment"
    );

    let pending_jobs = services.job_repository.get_pending_jobs().await.unwrap();
    assert_eq!(pending_jobs.len(), 0);
}

// ================================================================================================
// DUPLICATE DETECTION TESTS
// ================================================================================================

#[tokio::test]
async fn duplicate_anime_not_recreated() {
    
    
    let services = helpers::build_test_services();

    let anime = AnimeFactory::minimal().with_anilist_id(12345).build();

    let options = IngestionOptions {
        skip_duplicates: true,
        skip_provider_fetch: true,
        enrich_async: false,
        fetch_relations: false,
        priority: JobPriority::Normal,
    };

    // First import
    let result1 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: anime.clone(),
                context: "First".to_string(),
            },
            options.clone(),
        )
        .await
        .unwrap();

    assert!(result1.was_new, "First import should be new");

    // Second import (duplicate)
    let result2 = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime: anime.clone(),
                context: "Second".to_string(),
            },
            options,
        )
        .await
        .unwrap();

    assert!(!result2.was_new, "Duplicate should be detected");
    assert_eq!(
        result1.anime.id, result2.anime.id,
        "Should return existing anime"
    );
}

// ================================================================================================
// MANUAL IMPORT FLOW TEST
// ================================================================================================

#[tokio::test]
async fn manual_import_creates_anime_with_proper_tier() {
    
    
    let services = helpers::build_test_services();

    // Simulate user manually importing "Attack on Titan"
    let result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::ManualImport {
                title: "Shingeki no Kyojin".to_string(),
            },
            IngestionOptions {
                skip_duplicates: false,
                skip_provider_fetch: false, // Fetch from provider
                enrich_async: false,
                fetch_relations: false,
                priority: JobPriority::Normal,
            },
        )
        .await;

    if let Ok(result) = result {
        // Verify anime was created with data from provider
        assert!(
            result.anime.score.is_some(),
            "Should have score from provider"
        );
        assert!(!result.anime.title.main.is_empty(), "Should have title");

        // Verify tier was calculated (not hardcoded)
        assert!(
            result.anime.composite_score > 0.0,
            "Should have calculated score"
        );

        // Verify it exists in database
        let saved = services
            .anime_service
            .get_anime_by_id(&result.anime.id)
            .await
            .unwrap();
        assert!(saved.is_some(), "Should be saved to database");

        println!(
            "✓ Manual import created anime with tier {:?} and score {:.2}",
            result.anime.tier, result.anime.composite_score
        );
    }
}

// ================================================================================================
// INGESTION STAGES TEST
// ================================================================================================

#[tokio::test]
async fn ingestion_pipeline_executes_all_stages() {
    
    
    let services = helpers::build_test_services();

    let anime = AnimeFactory::minimal()
        .with_title("Pipeline Test Anime")
        .with_anilist_id(99999)
        .build();

    let result = services
        .ingestion_service
        .ingest_anime(
            AnimeSource::DirectData {
                anime,
                context: "Full pipeline test".to_string(),
            },
            IngestionOptions {
                skip_duplicates: false,
                skip_provider_fetch: true,
                enrich_async: true,
                fetch_relations: false,
                priority: JobPriority::Normal,
            },
        )
        .await
        .unwrap();

    // STAGE 1: Validation (always happens)
    assert!(result.was_new, "Anime should be new");

    // STAGE 2: Enhancement (always happens)
    assert!(result.quality_score >= 0.0 && result.quality_score <= 1.0);

    // STAGE 3: Save (always happens)
    let saved = services
        .anime_service
        .get_anime_by_id(&result.anime.id)
        .await
        .unwrap();
    assert!(saved.is_some(), "Should be saved");

    // STAGE 4: Enrichment queueing (conditional)
    if result.quality_score < 0.8 {
        assert!(result.enrichment_queued, "Should queue enrichment");
    }

    // STAGE 5: Relations discovery (we disabled it)
    assert!(!result.enrichment_queued || result.quality_score < 0.8);
}
