/// Diesel-based implementation of JobRepository
///
/// Uses PostgreSQL with SELECT FOR UPDATE SKIP LOCKED for atomic job dequeuing.
use crate::modules::jobs::domain::entities::{Job, JobRecord};
use crate::modules::jobs::domain::repository::{JobRepository, JobStatistics};
use crate::modules::jobs::infrastructure::models::{BackgroundJobModel, NewJob};
use crate::schema::background_jobs;
use crate::shared::errors::{AppError, AppResult};
use crate::shared::infrastructure::database::DbPool;
use async_trait::async_trait;
use diesel::prelude::*;
use uuid::Uuid;

/// Helper struct for COUNT queries
#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

pub struct JobRepositoryImpl {
    pool: DbPool,
}

impl JobRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get database connection from pool
    fn get_conn(
        &self,
    ) -> AppResult<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
    > {
        self.pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))
    }
}

#[async_trait]
impl JobRepository for JobRepositoryImpl {
    async fn enqueue(&self, job: Job) -> AppResult<JobRecord> {
        let new_job = NewJob {
            job_type: job.job_type.to_string(),
            payload: job.payload,
            priority: job.priority,
        };

        let mut conn = self.get_conn()?;

        let inserted: BackgroundJobModel = diesel::insert_into(background_jobs::table)
            .values(&new_job)
            .get_result(&mut conn)
            .map_err(|e| AppError::DatabaseError(format!("Failed to enqueue job: {}", e)))?;

        Ok(inserted.to_job_record())
    }

    async fn dequeue(&self) -> AppResult<Option<JobRecord>> {
        let mut conn = self.get_conn()?;

        // Atomic dequeue using SELECT FOR UPDATE SKIP LOCKED
        // This ensures no race conditions between multiple workers
        let result: Option<BackgroundJobModel> = diesel::sql_query(
            r#"
            UPDATE background_jobs
            SET status = 'running',
                started_at = NOW(),
                attempts = attempts + 1
            WHERE id = (
                SELECT id
                FROM background_jobs
                WHERE status = 'pending'
                  AND attempts < max_attempts
                ORDER BY priority ASC, created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, job_type, payload, priority, status,
                      attempts, max_attempts, created_at,
                      started_at, completed_at, error
            "#,
        )
        .get_result(&mut conn)
        .optional()
        .map_err(|e| AppError::DatabaseError(format!("Failed to dequeue job: {}", e)))?;

        Ok(result.map(|job| job.to_job_record()))
    }

    async fn mark_completed(&self, job_id: Uuid) -> AppResult<()> {
        let mut conn = self.get_conn()?;

        diesel::sql_query(
            "UPDATE background_jobs
             SET status = 'completed', completed_at = NOW()
             WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(job_id)
        .execute(&mut conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to mark job as completed: {}", e)))?;

        Ok(())
    }

    async fn mark_failed(&self, job_id: Uuid, error: &str) -> AppResult<()> {
        let mut conn = self.get_conn()?;

        // If attempts < max_attempts, reset to pending for retry
        // Otherwise, mark as permanently failed
        diesel::sql_query(
            "UPDATE background_jobs
             SET status = CASE
                 WHEN attempts < max_attempts THEN 'pending'::job_status
                 ELSE 'failed'::job_status
             END,
             completed_at = CASE
                 WHEN attempts >= max_attempts THEN NOW()
                 ELSE NULL
             END,
             started_at = NULL,
             error = $2
             WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(job_id)
        .bind::<diesel::sql_types::Text, _>(error)
        .execute(&mut conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to mark job as failed: {}", e)))?;

        Ok(())
    }

    async fn get_by_id(&self, job_id: Uuid) -> AppResult<Option<JobRecord>> {
        let mut conn = self.get_conn()?;

        let job: Option<BackgroundJobModel> = background_jobs::table
            .find(job_id)
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get job by id: {}", e)))?;

        Ok(job.map(|j| j.to_job_record()))
    }

    async fn get_pending_jobs(&self) -> AppResult<Vec<JobRecord>> {
        let mut conn = self.get_conn()?;

        let jobs: Vec<BackgroundJobModel> = diesel::sql_query(
            "SELECT id, job_type, payload, priority, status,
                    attempts, max_attempts, created_at,
                    started_at, completed_at, error
             FROM background_jobs
             WHERE status = 'pending'
             ORDER BY priority ASC, created_at ASC",
        )
        .load(&mut conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to get pending jobs: {}", e)))?;

        Ok(jobs.into_iter().map(|j| j.to_job_record()).collect())
    }

    async fn get_jobs_for_anime(&self, anime_id: Uuid) -> AppResult<Vec<JobRecord>> {
        let mut conn = self.get_conn()?;

        // Query JSONB payload for anime_id
        let jobs: Vec<BackgroundJobModel> = diesel::sql_query(
            "SELECT id, job_type, payload, priority, status,
                    attempts, max_attempts, created_at,
                    started_at, completed_at, error
             FROM background_jobs
             WHERE payload->>'anime_id' = $1
             ORDER BY created_at DESC",
        )
        .bind::<diesel::sql_types::Text, _>(anime_id.to_string())
        .load(&mut conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to get jobs for anime: {}", e)))?;

        Ok(jobs.into_iter().map(|j| j.to_job_record()).collect())
    }

    async fn delete_old_completed(&self, days: i32) -> AppResult<usize> {
        let mut conn = self.get_conn()?;

        let deleted = diesel::sql_query(
            "DELETE FROM background_jobs
             WHERE status IN ('completed', 'failed')
             AND completed_at < NOW() - INTERVAL '1 day' * $1",
        )
        .bind::<diesel::sql_types::Integer, _>(days)
        .execute(&mut conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete old jobs: {}", e)))?;

        Ok(deleted)
    }

    async fn get_statistics(&self) -> AppResult<JobStatistics> {
        let mut conn = self.get_conn()?;

        // Use raw SQL for counting different statuses
        let pending: CountResult = diesel::sql_query(
            "SELECT COUNT(*) as count FROM background_jobs WHERE status = 'pending'",
        )
        .get_result(&mut conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to count pending: {}", e)))?;

        let running: CountResult = diesel::sql_query(
            "SELECT COUNT(*) as count FROM background_jobs WHERE status = 'running'",
        )
        .get_result(&mut conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to count running: {}", e)))?;

        let completed: CountResult = diesel::sql_query(
            "SELECT COUNT(*) as count FROM background_jobs WHERE status = 'completed'",
        )
        .get_result(&mut conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to count completed: {}", e)))?;

        let failed: CountResult = diesel::sql_query(
            "SELECT COUNT(*) as count FROM background_jobs WHERE status = 'failed'",
        )
        .get_result(&mut conn)
        .map_err(|e| AppError::DatabaseError(format!("Failed to count failed: {}", e)))?;

        let total: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM background_jobs")
            .get_result(&mut conn)
            .map_err(|e| AppError::DatabaseError(format!("Failed to count total: {}", e)))?;

        Ok(JobStatistics {
            pending_count: pending.count,
            running_count: running.count,
            completed_count: completed.count,
            failed_count: failed.count,
            total_count: total.count,
        })
    }
}
