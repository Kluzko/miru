/// Repository trait for job persistence
///
/// Defines the interface for job storage and retrieval operations.
/// Implementation will use Diesel ORM with PostgreSQL.
use crate::modules::jobs::domain::entities::{Job, JobRecord};
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait JobRepository: Send + Sync {
    /// Enqueue a new job
    async fn enqueue(&self, job: Job) -> AppResult<JobRecord>;

    /// Dequeue the next pending job (atomic operation using SELECT FOR UPDATE SKIP LOCKED)
    /// Returns None if no jobs are available
    async fn dequeue(&self) -> AppResult<Option<JobRecord>>;

    /// Mark job as completed
    async fn mark_completed(&self, job_id: Uuid) -> AppResult<()>;

    /// Mark job as failed with error message
    async fn mark_failed(&self, job_id: Uuid, error: &str) -> AppResult<()>;

    /// Get job by ID
    async fn get_by_id(&self, job_id: Uuid) -> AppResult<Option<JobRecord>>;

    /// Get all pending jobs (for monitoring)
    async fn get_pending_jobs(&self) -> AppResult<Vec<JobRecord>>;

    /// Get all jobs for a specific anime (for UI progress tracking)
    async fn get_jobs_for_anime(&self, anime_id: Uuid) -> AppResult<Vec<JobRecord>>;

    /// Delete old completed jobs (cleanup)
    async fn delete_old_completed(&self, days: i32) -> AppResult<usize>;

    /// Get job statistics
    async fn get_statistics(&self) -> AppResult<JobStatistics>;
}

/// Job queue statistics
#[derive(Debug, Clone)]
pub struct JobStatistics {
    pub pending_count: i64,
    pub running_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub total_count: i64,
}
