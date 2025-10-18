use super::database::Database;
use crate::shared::errors::AppError;
use std::sync::Arc;

/// Represents the state of the database connection
/// This allows the application to gracefully handle database failures
/// without terminating the entire application
#[derive(Debug, Clone)]
pub enum DatabaseState {
    /// Database is available and ready for use
    Available(Arc<Database>),
    /// Database is unavailable with the reason for failure
    Unavailable {
        reason: String,
        last_attempt: std::time::Instant,
        retry_count: u32,
    },
    /// Database is being initialized or reconnected
    Initializing,
}

impl DatabaseState {
    /// Create a new database state by attempting to initialize the database
    pub fn initialize() -> Self {
        match Database::new() {
            Ok(db) => {
                log::info!("Database initialized successfully");
                DatabaseState::Available(Arc::new(db))
            }
            Err(e) => {
                log::error!("Database initialization failed: {}", e);
                DatabaseState::Unavailable {
                    reason: e.to_string(),
                    last_attempt: std::time::Instant::now(),
                    retry_count: 0,
                }
            }
        }
    }

    /// Check if the database is available
    pub fn is_available(&self) -> bool {
        matches!(self, DatabaseState::Available(_))
    }

    /// Get the database if available, otherwise return an error
    pub fn get_database(&self) -> Result<Arc<Database>, AppError> {
        match self {
            DatabaseState::Available(db) => Ok(Arc::clone(db)),
            DatabaseState::Unavailable { reason, .. } => Err(AppError::ServiceUnavailable(
                format!("Database unavailable: {}", reason),
            )),
            DatabaseState::Initializing => Err(AppError::ServiceUnavailable(
                "Database is initializing, please try again".to_string(),
            )),
        }
    }

    /// Attempt to reconnect to the database if it's currently unavailable
    /// This implements exponential backoff to avoid overwhelming the database
    pub fn attempt_reconnect(&mut self) -> bool {
        match self {
            DatabaseState::Unavailable {
                last_attempt,
                retry_count,
                ..
            } => {
                // Exponential backoff: wait 2^retry_count seconds, max 5 minutes
                let current_retry_count = *retry_count;
                let backoff_duration = std::time::Duration::from_secs(std::cmp::min(
                    2_u64.pow(current_retry_count),
                    300,
                ));

                if last_attempt.elapsed() < backoff_duration {
                    return false; // Too soon to retry
                }

                *self = DatabaseState::Initializing;

                match Database::new() {
                    Ok(db) => {
                        log::info!(
                            "Database reconnection successful after {} attempts",
                            current_retry_count + 1
                        );
                        *self = DatabaseState::Available(Arc::new(db));
                        true
                    }
                    Err(e) => {
                        log::warn!(
                            "Database reconnection attempt {} failed: {}",
                            current_retry_count + 1,
                            e
                        );
                        *self = DatabaseState::Unavailable {
                            reason: e.to_string(),
                            last_attempt: std::time::Instant::now(),
                            retry_count: current_retry_count + 1,
                        };
                        false
                    }
                }
            }
            DatabaseState::Available(_) => true,
            DatabaseState::Initializing => false,
        }
    }

    /// Get a user-friendly status message for the database state
    pub fn status_message(&self) -> String {
        match self {
            DatabaseState::Available(_) => "Database connected".to_string(),
            DatabaseState::Unavailable {
                reason,
                retry_count,
                ..
            } => {
                format!("Database unavailable (attempt {}): {}", retry_count, reason)
            }
            DatabaseState::Initializing => "Database connecting...".to_string(),
        }
    }
}

/// Background task to periodically attempt database reconnection
pub struct DatabaseHealthMonitor {
    state: Arc<tokio::sync::RwLock<DatabaseState>>,
}

impl DatabaseHealthMonitor {
    pub fn new(initial_state: DatabaseState) -> Self {
        Self {
            state: Arc::new(tokio::sync::RwLock::new(initial_state)),
        }
    }

    pub fn get_state(&self) -> Arc<tokio::sync::RwLock<DatabaseState>> {
        Arc::clone(&self.state)
    }

    /// Start the background health monitoring task
    pub async fn start_monitoring(&self) {
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                let mut db_state = state.write().await;
                if !db_state.is_available() {
                    if db_state.attempt_reconnect() {
                        log::info!("Database reconnection successful");
                    }
                }
            }
        });
    }
}
