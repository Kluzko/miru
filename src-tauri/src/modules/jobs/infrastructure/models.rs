/// Diesel models for background_jobs table
use crate::modules::jobs::domain::value_objects::JobStatusDb;
use crate::schema::background_jobs;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Diesel model for inserting new jobs
#[derive(Insertable, Debug)]
#[diesel(table_name = background_jobs)]
pub struct NewJob {
    pub job_type: String,
    pub payload: JsonValue,
    pub priority: i32,
}

/// Diesel model for querying existing jobs
#[derive(Queryable, Selectable, QueryableByName, Debug, Clone)]
#[diesel(table_name = background_jobs)]
pub struct BackgroundJobModel {
    pub id: Uuid,
    pub job_type: String,
    pub payload: JsonValue,
    pub priority: i32,
    pub status: JobStatusDb,
    pub attempts: i32,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl BackgroundJobModel {
    /// Convert to domain JobRecord
    pub fn to_job_record(self) -> crate::modules::jobs::domain::entities::JobRecord {
        crate::modules::jobs::domain::entities::JobRecord {
            id: self.id,
            job_type: self.job_type,
            payload: self.payload,
            priority: self.priority,
            status: self.status.to_string(),
            attempts: self.attempts,
            max_attempts: self.max_attempts,
            created_at: self.created_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            error: self.error,
        }
    }
}
