/// Job repository tests - database operations
///
/// Tests cover:
/// - Basic CRUD operations
/// - Job state transitions
/// - Atomic dequeue operations
/// - Query filtering
mod utils;

use miru_lib::modules::jobs::domain::{entities::Job, repository::JobRepository};
use utils::db;

#[tokio::test]
async fn enqueue_and_retrieve_job() {
    let _guard = db::acquire_test_lock();
    db::clean_test_db();

    let pool = db::get_test_db_pool();
    let repo = miru_lib::modules::jobs::infrastructure::JobRepositoryImpl::new((*pool).clone());

    let anime_id = uuid::Uuid::new_v4();
    let job = Job::enrichment(anime_id, 5);

    let enqueued = repo.enqueue(job).await.unwrap();
    assert_eq!(enqueued.job_type, "enrichment");
    assert_eq!(enqueued.status, "pending");
    assert_eq!(enqueued.priority, 5);

    let retrieved = repo.get_by_id(enqueued.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, enqueued.id);
}

#[tokio::test]
async fn dequeue_returns_pending_job() {
    let _guard = db::acquire_test_lock();
    db::clean_test_db();

    let pool = db::get_test_db_pool();
    let repo = miru_lib::modules::jobs::infrastructure::JobRepositoryImpl::new((*pool).clone());

    let anime_id = uuid::Uuid::new_v4();
    repo.enqueue(Job::enrichment(anime_id, 5)).await.unwrap();

    let dequeued = repo.dequeue().await.unwrap();
    assert!(dequeued.is_some());

    let job = dequeued.unwrap();
    assert_eq!(job.status, "running");
    assert_eq!(job.attempts, 1);
}

#[tokio::test]
async fn dequeue_empty_queue_returns_none() {
    let _guard = db::acquire_test_lock();
    db::clean_test_db();

    let pool = db::get_test_db_pool();
    let repo = miru_lib::modules::jobs::infrastructure::JobRepositoryImpl::new((*pool).clone());

    let result = repo.dequeue().await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn mark_completed_updates_status() {
    let _guard = db::acquire_test_lock();
    db::clean_test_db();

    let pool = db::get_test_db_pool();
    let repo = miru_lib::modules::jobs::infrastructure::JobRepositoryImpl::new((*pool).clone());

    let anime_id = uuid::Uuid::new_v4();
    let enqueued = repo.enqueue(Job::enrichment(anime_id, 5)).await.unwrap();
    let job_id = enqueued.id;

    repo.dequeue().await.unwrap();
    repo.mark_completed(job_id).await.unwrap();

    let job = repo.get_by_id(job_id).await.unwrap().unwrap();
    assert_eq!(job.status, "completed");
    assert!(job.completed_at.is_some());
}

#[tokio::test]
async fn mark_failed_with_retries() {
    let _guard = db::acquire_test_lock();
    db::clean_test_db();

    let pool = db::get_test_db_pool();
    let repo = miru_lib::modules::jobs::infrastructure::JobRepositoryImpl::new((*pool).clone());

    let anime_id = uuid::Uuid::new_v4();
    let enqueued = repo.enqueue(Job::enrichment(anime_id, 5)).await.unwrap();
    let job_id = enqueued.id;

    // Fail first attempt
    repo.dequeue().await.unwrap();
    repo.mark_failed(job_id, "Test error").await.unwrap();

    let job = repo.get_by_id(job_id).await.unwrap().unwrap();
    assert_eq!(job.status, "pending"); // Reset for retry
    assert_eq!(job.attempts, 1);
    assert!(job.error.is_some());
}

#[tokio::test]
async fn get_pending_jobs_filters_correctly() {
    let _guard = db::acquire_test_lock();
    db::clean_test_db();

    let pool = db::get_test_db_pool();
    let repo = miru_lib::modules::jobs::infrastructure::JobRepositoryImpl::new((*pool).clone());

    // Create 3 pending jobs
    for i in 0..3 {
        let anime_id = uuid::Uuid::new_v4();
        repo.enqueue(Job::enrichment(anime_id, i + 1))
            .await
            .unwrap();
    }

    let pending = repo.get_pending_jobs().await.unwrap();
    assert_eq!(pending.len(), 3);
    assert!(pending.iter().all(|j| j.status == "pending"));
}

#[tokio::test]
async fn get_jobs_for_anime_filters_correctly() {
    let _guard = db::acquire_test_lock();
    db::clean_test_db();

    let pool = db::get_test_db_pool();
    let repo = miru_lib::modules::jobs::infrastructure::JobRepositoryImpl::new((*pool).clone());

    let anime_id = uuid::Uuid::new_v4();
    let other_anime_id = uuid::Uuid::new_v4();

    repo.enqueue(Job::enrichment(anime_id, 5)).await.unwrap();
    repo.enqueue(Job::enrichment(anime_id, 5)).await.unwrap();
    repo.enqueue(Job::enrichment(other_anime_id, 5))
        .await
        .unwrap();

    let jobs = repo.get_jobs_for_anime(anime_id).await.unwrap();
    assert_eq!(jobs.len(), 2);
    assert!(jobs.iter().all(|j| {
        let payload = j.parse_enrichment_payload().unwrap();
        payload.anime_id == anime_id
    }));
}

#[tokio::test]
async fn priority_ordering_works() {
    let _guard = db::acquire_test_lock();
    db::clean_test_db();

    let pool = db::get_test_db_pool();
    let repo = miru_lib::modules::jobs::infrastructure::JobRepositoryImpl::new((*pool).clone());

    repo.enqueue(Job::enrichment(uuid::Uuid::new_v4(), 10))
        .await
        .unwrap();
    repo.enqueue(Job::enrichment(uuid::Uuid::new_v4(), 1))
        .await
        .unwrap();
    repo.enqueue(Job::enrichment(uuid::Uuid::new_v4(), 5))
        .await
        .unwrap();

    let job1 = repo.dequeue().await.unwrap().unwrap();
    assert_eq!(job1.priority, 1);

    let job2 = repo.dequeue().await.unwrap().unwrap();
    assert_eq!(job2.priority, 5);

    let job3 = repo.dequeue().await.unwrap().unwrap();
    assert_eq!(job3.priority, 10);
}

#[tokio::test]
async fn payload_serialization_roundtrip() {
    let _guard = db::acquire_test_lock();
    db::clean_test_db();

    let pool = db::get_test_db_pool();
    let repo = miru_lib::modules::jobs::infrastructure::JobRepositoryImpl::new((*pool).clone());

    let anime_id = uuid::Uuid::new_v4();
    let job = Job::enrichment(anime_id, 5);
    let enqueued = repo.enqueue(job).await.unwrap();

    let payload = enqueued.parse_enrichment_payload().unwrap();
    assert_eq!(payload.anime_id, anime_id);

    // Test relations payload
    let job2 = Job::relations_discovery(anime_id, 5);
    let enqueued2 = repo.enqueue(job2).await.unwrap();

    let payload2 = enqueued2.parse_relations_payload().unwrap();
    assert_eq!(payload2.anime_id, anime_id);
}
