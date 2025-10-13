/// Domain entities for background job system
///
/// Jobs represent async tasks like anime enrichment and relations discovery
/// that can be queued and processed by background workers.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job status enum matching database type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(JobStatus::Pending),
            "running" => Ok(JobStatus::Running),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            _ => Err(format!("Invalid job status: {}", s)),
        }
    }
}

/// Job type enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Enrichment,
    RelationsDiscovery,
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::Enrichment => write!(f, "enrichment"),
            JobType::RelationsDiscovery => write!(f, "relations_discovery"),
        }
    }
}

impl std::str::FromStr for JobType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "enrichment" => Ok(JobType::Enrichment),
            "relations_discovery" => Ok(JobType::RelationsDiscovery),
            _ => Err(format!("Invalid job type: {}", s)),
        }
    }
}

/// Job payload for enrichment jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentJobPayload {
    pub anime_id: Uuid,
}

/// Job payload for relations discovery jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationsDiscoveryJobPayload {
    pub anime_id: Uuid,
}

/// New job to be queued (before insertion to database)
#[derive(Debug, Clone)]
pub struct Job {
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub priority: i32,
}

impl Job {
    /// Create a new enrichment job
    pub fn enrichment(anime_id: Uuid, priority: i32) -> Self {
        let payload = EnrichmentJobPayload { anime_id };
        Self {
            job_type: JobType::Enrichment,
            payload: serde_json::to_value(payload).unwrap(),
            priority,
        }
    }

    /// Create a new relations discovery job
    pub fn relations_discovery(anime_id: Uuid, priority: i32) -> Self {
        let payload = RelationsDiscoveryJobPayload { anime_id };
        Self {
            job_type: JobType::RelationsDiscovery,
            payload: serde_json::to_value(payload).unwrap(),
            priority,
        }
    }
}

/// Job record from database (with metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl JobRecord {
    /// Parse job type
    pub fn parse_job_type(&self) -> Result<JobType, String> {
        self.job_type.parse()
    }

    /// Parse job status
    pub fn parse_status(&self) -> Result<JobStatus, String> {
        self.status.parse()
    }

    /// Check if job can be retried
    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }

    /// Parse enrichment payload
    pub fn parse_enrichment_payload(&self) -> Result<EnrichmentJobPayload, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }

    /// Parse relations discovery payload
    pub fn parse_relations_payload(
        &self,
    ) -> Result<RelationsDiscoveryJobPayload, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Pending.to_string(), "pending");
        assert_eq!(JobStatus::Running.to_string(), "running");
        assert_eq!(JobStatus::Completed.to_string(), "completed");
        assert_eq!(JobStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_job_status_from_str() {
        assert_eq!("pending".parse::<JobStatus>().unwrap(), JobStatus::Pending);
        assert_eq!("RUNNING".parse::<JobStatus>().unwrap(), JobStatus::Running);
        assert!("invalid".parse::<JobStatus>().is_err());
    }

    #[test]
    fn test_job_type_display() {
        assert_eq!(JobType::Enrichment.to_string(), "enrichment");
        assert_eq!(
            JobType::RelationsDiscovery.to_string(),
            "relations_discovery"
        );
    }

    #[test]
    fn test_create_enrichment_job() {
        let anime_id = Uuid::new_v4();
        let job = Job::enrichment(anime_id, 5);

        assert_eq!(job.job_type, JobType::Enrichment);
        assert_eq!(job.priority, 5);

        let payload: EnrichmentJobPayload = serde_json::from_value(job.payload).unwrap();
        assert_eq!(payload.anime_id, anime_id);
    }

    #[test]
    fn test_create_relations_job() {
        let anime_id = Uuid::new_v4();
        let job = Job::relations_discovery(anime_id, 3);

        assert_eq!(job.job_type, JobType::RelationsDiscovery);
        assert_eq!(job.priority, 3);
    }

    #[test]
    fn test_job_record_can_retry() {
        use chrono::Utc;

        let job = JobRecord {
            id: Uuid::new_v4(),
            job_type: "enrichment".to_string(),
            payload: serde_json::json!({"anime_id": Uuid::new_v4()}),
            priority: 5,
            status: "failed".to_string(),
            attempts: 2,
            max_attempts: 3,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: Some("Test error".to_string()),
        };

        assert!(
            job.can_retry(),
            "Should be able to retry when attempts < max_attempts"
        );

        let exhausted = JobRecord { attempts: 3, ..job };

        assert!(
            !exhausted.can_retry(),
            "Should not retry when attempts >= max_attempts"
        );
    }

    #[test]
    fn test_job_record_parse_payloads() {
        use chrono::Utc;

        let anime_id = Uuid::new_v4();

        let enrichment_job = JobRecord {
            id: Uuid::new_v4(),
            job_type: "enrichment".to_string(),
            payload: serde_json::json!({"anime_id": anime_id}),
            priority: 5,
            status: "pending".to_string(),
            attempts: 0,
            max_attempts: 3,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        };

        let payload = enrichment_job.parse_enrichment_payload().unwrap();
        assert_eq!(payload.anime_id, anime_id);

        let relations_job = JobRecord {
            job_type: "relations_discovery".to_string(),
            payload: serde_json::json!({"anime_id": anime_id}),
            ..enrichment_job.clone()
        };

        let payload = relations_job.parse_relations_payload().unwrap();
        assert_eq!(payload.anime_id, anime_id);
    }
}
