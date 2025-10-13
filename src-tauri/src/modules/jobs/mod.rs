/// Background job system module
///
/// Provides a PostgreSQL-based job queue for async operations like:
/// - Anime enrichment (fetching missing data from providers)
/// - Relations discovery (finding and ingesting related anime)
///
/// Architecture:
/// - Domain: Entities and repository trait
/// - Infrastructure: Diesel-based repository implementation
/// - Worker: Background worker that processes jobs
pub mod domain;
pub mod infrastructure;
pub mod worker;

// Re-exports for easy access
pub use domain::{
    entities::{
        EnrichmentJobPayload, Job, JobRecord, JobStatus, JobType, RelationsDiscoveryJobPayload,
    },
    repository::{JobRepository, JobStatistics},
};
pub use infrastructure::JobRepositoryImpl;
pub use worker::{BackgroundWorker, WorkerStatistics};
