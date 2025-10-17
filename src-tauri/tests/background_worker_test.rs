/// Comprehensive tests for background job processing
///
/// Tests cover:
/// - Job processing and completion
/// - Priority-based job ordering
/// - Retry logic (3 attempts then fail)
/// - Worker lifecycle (start/stop)
mod utils;

use miru_lib::modules::jobs::domain::{entities::Job, repository::JobRepository};
use utils::{factories::AnimeFactory, helpers};

// ================================================================================================
// JOB PROCESSING TESTS
// ================================================================================================

#[tokio::test]
async fn worker_processes_single_job() {
    
    
    let services = helpers::build_test_services();

    // Create anime and enqueue job
    let anime = AnimeFactory::minimal().with_anilist_id(16498).build();
    services.anime_service.create_anime(&anime).await.unwrap();

    let job = Job::enrichment(anime.id, 5);
    let job_record = services.job_repository.enqueue(job).await.unwrap();
    let job_id = job_record.id;

    // Verify job is pending
    let pending = services
        .job_repository
        .get_pending_jobs()
        .await
        .unwrap()
        .len();
    assert_eq!(pending, 1);

    // Run worker
    helpers::run_worker_for_duration(services.background_worker.clone(), 3).await;

    // Verify job was processed
    let pending_after = services
        .job_repository
        .get_pending_jobs()
        .await
        .unwrap()
        .len();
    assert_eq!(pending_after, 0, "Job should be processed");

    let final_job = services.job_repository.get_by_id(job_id).await.unwrap();
    assert!(final_job.is_some());
    let status = final_job.unwrap().status;
    assert!(
        status == "completed" || status == "failed",
        "Job should be completed or failed, got: {}",
        status
    );
}

#[tokio::test]
async fn worker_processes_multiple_jobs() {
    
    
    let services = helpers::build_test_services();

    // Create 5 anime and queue jobs
    for i in 0..5 {
        let anime = AnimeFactory::minimal().with_anilist_id(10000 + i).build();
        services.anime_service.create_anime(&anime).await.unwrap();
        services
            .job_repository
            .enqueue(Job::enrichment(anime.id, 5))
            .await
            .unwrap();
    }

    let pending_before = services
        .job_repository
        .get_pending_jobs()
        .await
        .unwrap()
        .len();
    assert_eq!(pending_before, 5);

    // Run worker
    helpers::run_worker_for_duration(services.background_worker.clone(), 6).await;

    // All jobs should be processed
    let pending_after = services
        .job_repository
        .get_pending_jobs()
        .await
        .unwrap()
        .len();
    assert_eq!(pending_after, 0, "All jobs should be processed");
}

// ================================================================================================
// PRIORITY TESTS
// ================================================================================================

#[tokio::test]
async fn jobs_processed_by_priority() {
    
    
    let services = helpers::build_test_services();

    // Create anime
    let anime1 = AnimeFactory::minimal().with_anilist_id(20001).build();
    let anime2 = AnimeFactory::minimal().with_anilist_id(20002).build();
    let anime3 = AnimeFactory::minimal().with_anilist_id(20003).build();

    services.anime_service.create_anime(&anime1).await.unwrap();
    services.anime_service.create_anime(&anime2).await.unwrap();
    services.anime_service.create_anime(&anime3).await.unwrap();

    // Enqueue with different priorities
    services
        .job_repository
        .enqueue(Job::enrichment(anime1.id, 10))
        .await
        .unwrap(); // Low priority
    services
        .job_repository
        .enqueue(Job::enrichment(anime2.id, 1))
        .await
        .unwrap(); // High priority
    services
        .job_repository
        .enqueue(Job::enrichment(anime3.id, 5))
        .await
        .unwrap(); // Normal priority

    // Dequeue and verify order
    let job1 = services.job_repository.dequeue().await.unwrap().unwrap();
    assert_eq!(job1.priority, 1, "High priority should be first");

    let job2 = services.job_repository.dequeue().await.unwrap().unwrap();
    assert_eq!(job2.priority, 5, "Normal priority should be second");

    let job3 = services.job_repository.dequeue().await.unwrap().unwrap();
    assert_eq!(job3.priority, 10, "Low priority should be last");
}

// ================================================================================================
// RETRY LOGIC TESTS
// ================================================================================================

#[tokio::test]
async fn failed_job_retries_up_to_3_times() {
    
    
    let services = helpers::build_test_services();

    // Create job for non-existent anime (will fail)
    let fake_anime_id = uuid::Uuid::new_v4();
    let job = Job::enrichment(fake_anime_id, 5);
    let enqueued = services.job_repository.enqueue(job).await.unwrap();

    // Attempt 1
    let job1 = services.job_repository.dequeue().await.unwrap().unwrap();
    assert_eq!(job1.attempts, 1);
    services
        .job_repository
        .mark_failed(job1.id, "Anime not found")
        .await
        .unwrap();

    let after_fail1 = services
        .job_repository
        .get_by_id(enqueued.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        after_fail1.status, "pending",
        "Should retry after first failure"
    );
    assert_eq!(after_fail1.attempts, 1);

    // Attempt 2
    let job2 = services.job_repository.dequeue().await.unwrap().unwrap();
    assert_eq!(job2.attempts, 2);
    services
        .job_repository
        .mark_failed(job2.id, "Anime not found")
        .await
        .unwrap();

    let after_fail2 = services
        .job_repository
        .get_by_id(enqueued.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        after_fail2.status, "pending",
        "Should retry after second failure"
    );

    // Attempt 3 (final)
    let job3 = services.job_repository.dequeue().await.unwrap().unwrap();
    assert_eq!(job3.attempts, 3);
    services
        .job_repository
        .mark_failed(job3.id, "Anime not found")
        .await
        .unwrap();

    let final_job = services
        .job_repository
        .get_by_id(enqueued.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        final_job.status, "failed",
        "Should be permanently failed after 3 attempts"
    );
    assert_eq!(final_job.attempts, 3);

    // Should not be dequeueable anymore
    let next = services.job_repository.dequeue().await.unwrap();
    assert!(next.is_none(), "Failed job should not be dequeueable");
}

// ================================================================================================
// RELATIONS DISCOVERY JOB TEST
// ================================================================================================

#[tokio::test]
async fn relations_discovery_job_can_be_queued() {
    
    
    let services = helpers::build_test_services();

    let anime = AnimeFactory::minimal().with_anilist_id(16498).build();
    services.anime_service.create_anime(&anime).await.unwrap();

    // Queue relations discovery job
    let job = Job::relations_discovery(anime.id, 5);
    let job_record = services.job_repository.enqueue(job).await.unwrap();

    assert_eq!(job_record.job_type, "relations_discovery");

    let payload = job_record.parse_relations_payload().unwrap();
    assert_eq!(payload.anime_id, anime.id);
}

// ================================================================================================
// ATOMIC OPERATIONS TEST
// ================================================================================================

#[tokio::test]
async fn concurrent_dequeue_no_race_condition() {
    
    
    let services = helpers::build_test_services();

    // Create and enqueue 10 jobs
    for i in 0..10 {
        let anime = AnimeFactory::minimal().with_anilist_id(30000 + i).build();
        services.anime_service.create_anime(&anime).await.unwrap();
        services
            .job_repository
            .enqueue(Job::enrichment(anime.id, 5))
            .await
            .unwrap();
    }

    // Spawn 5 concurrent dequeuers
    let mut handles = vec![];
    for _ in 0..5 {
        let repo = services.job_repository.clone();
        let handle = tokio::spawn(async move {
            let mut dequeued = Vec::new();
            while let Ok(Some(job)) = repo.dequeue().await {
                dequeued.push(job.id);
            }
            dequeued
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut all_dequeued = Vec::new();
    for handle in handles {
        let jobs = handle.await.unwrap();
        all_dequeued.extend(jobs);
    }

    // Should have dequeued exactly 10 jobs (no duplicates)
    assert_eq!(
        all_dequeued.len(),
        10,
        "Should dequeue all jobs exactly once"
    );

    // Verify no duplicates
    all_dequeued.sort();
    all_dequeued.dedup();
    assert_eq!(all_dequeued.len(), 10, "Should have no duplicate job IDs");
}

// ================================================================================================
// JOB STATISTICS TEST
// ================================================================================================

#[tokio::test]
async fn job_statistics_accurate() {
    
    
    let services = helpers::build_test_services();

    // Create 3 anime
    let anime1 = AnimeFactory::minimal().with_anilist_id(40001).build();
    let anime2 = AnimeFactory::minimal().with_anilist_id(40002).build();
    let anime3 = AnimeFactory::minimal().with_anilist_id(40003).build();

    services.anime_service.create_anime(&anime1).await.unwrap();
    services.anime_service.create_anime(&anime2).await.unwrap();
    services.anime_service.create_anime(&anime3).await.unwrap();

    // Queue 3 jobs
    services
        .job_repository
        .enqueue(Job::enrichment(anime1.id, 5))
        .await
        .unwrap();
    services
        .job_repository
        .enqueue(Job::enrichment(anime2.id, 5))
        .await
        .unwrap();
    services
        .job_repository
        .enqueue(Job::enrichment(anime3.id, 5))
        .await
        .unwrap();

    let stats = services.job_repository.get_statistics().await.unwrap();
    assert_eq!(stats.total_count, 3);
    assert_eq!(stats.pending_count, 3);
    assert_eq!(stats.running_count, 0);
    assert_eq!(stats.completed_count, 0);
    assert_eq!(stats.failed_count, 0);

    // Process one job
    let job = services.job_repository.dequeue().await.unwrap().unwrap();
    services
        .job_repository
        .mark_completed(job.id)
        .await
        .unwrap();

    let stats_after = services.job_repository.get_statistics().await.unwrap();
    assert_eq!(stats_after.total_count, 3);
    assert_eq!(stats_after.pending_count, 2);
    assert_eq!(stats_after.completed_count, 1);
}
