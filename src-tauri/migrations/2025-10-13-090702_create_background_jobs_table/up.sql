-- Create background_jobs table for desktop-appropriate job queue
-- Uses PostgreSQL for persistence (no Redis/RabbitMQ needed for desktop app)

CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed', 'failed');

CREATE TABLE background_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    priority INTEGER NOT NULL DEFAULT 5,
    status job_status NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error TEXT,
    CONSTRAINT valid_priority CHECK (priority BETWEEN 1 AND 10)
);

-- Index for efficient job polling (most important query)
-- Worker will SELECT pending jobs ordered by priority, created_at
CREATE INDEX idx_jobs_pending ON background_jobs(status, priority DESC, created_at ASC)
WHERE status = 'pending';

-- Index for finding jobs by anime_id (for UI progress tracking)
-- Payload is JSONB, so we can use GIN index for efficient queries
CREATE INDEX idx_jobs_payload ON background_jobs USING GIN (payload);

-- Index for cleanup queries (finding old completed/failed jobs)
CREATE INDEX idx_jobs_completed_at ON background_jobs(completed_at)
WHERE completed_at IS NOT NULL;

-- Comments for documentation
COMMENT ON TABLE background_jobs IS 'Background job queue for async anime enrichment and relations discovery';
COMMENT ON COLUMN background_jobs.job_type IS 'Job type: enrichment, relations_discovery';
COMMENT ON COLUMN background_jobs.payload IS 'JSON payload with job-specific data (anime_id, etc.)';
COMMENT ON COLUMN background_jobs.priority IS 'Priority 1-10 (1=highest, 10=lowest)';
COMMENT ON COLUMN background_jobs.attempts IS 'Number of execution attempts';
COMMENT ON COLUMN background_jobs.max_attempts IS 'Maximum retry attempts before marking as failed';
