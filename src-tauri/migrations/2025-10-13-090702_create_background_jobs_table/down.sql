-- Drop background_jobs table and related objects

DROP INDEX IF EXISTS idx_jobs_completed_at;
DROP INDEX IF EXISTS idx_jobs_payload;
DROP INDEX IF EXISTS idx_jobs_pending;

DROP TABLE IF EXISTS background_jobs;

DROP TYPE IF EXISTS job_status;
