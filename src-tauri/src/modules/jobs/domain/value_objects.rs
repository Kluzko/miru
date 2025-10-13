/// Value objects for jobs domain
use serde::{Deserialize, Serialize};

/// Job status enum matching database type
#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    specta::Type,
)]
#[ExistingTypePath = "crate::schema::sql_types::JobStatus"]
#[serde(rename_all = "lowercase")]
pub enum JobStatusDb {
    Pending,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for JobStatusDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatusDb::Pending => write!(f, "pending"),
            JobStatusDb::Running => write!(f, "running"),
            JobStatusDb::Completed => write!(f, "completed"),
            JobStatusDb::Failed => write!(f, "failed"),
        }
    }
}
