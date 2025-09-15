use crate::log_info;
use crate::shared::errors::AppError;
use crate::shared::utils::logger::LogContext;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};
use std::env;
use std::time::Duration;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new() -> Result<Self, AppError> {
        let database_url = env::var("DATABASE_URL")?;

        // Enhanced connection pool configuration
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        let pool = r2d2::Pool::builder()
            // Pool sizing - optimized for desktop app with moderate concurrent usage
            .max_size(20) // Maximum connections in pool
            .min_idle(Some(3)) // Minimum idle connections to maintain
            // Connection timeouts
            .connection_timeout(Duration::from_secs(10)) // Time to wait for connection from pool
            .idle_timeout(Some(Duration::from_secs(300))) // Close idle connections after 5 minutes
            .max_lifetime(Some(Duration::from_secs(1800))) // Replace connections after 30 minutes
            // Connection health checks
            .test_on_check_out(true) // Test connections when borrowed from pool
            // Build the pool
            .build(manager)
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to create connection pool: {}", e))
            })?;

        log_info!(
            "Database connection pool initialized with max_size: {}, min_idle: {:?}",
            pool.max_size(),
            3
        );

        Ok(Self { pool })
    }

    pub fn get_connection(&self) -> Result<DbConnection, AppError> {
        let start = std::time::Instant::now();

        match self.pool.get() {
            Ok(conn) => {
                let duration = start.elapsed().as_millis() as u64;
                if duration > 100 {
                    LogContext::performance_metric("db_connection_acquire", duration, Some("slow"));
                }
                Ok(conn)
            }
            Err(e) => {
                LogContext::error_with_context(
                    &e,
                    "Failed to acquire database connection from pool",
                );
                Err(AppError::from(e))
            }
        }
    }

    /// Get pool statistics for monitoring
    pub fn pool_status(&self) -> PoolStatus {
        let state = self.pool.state();
        PoolStatus {
            connections: state.connections,
            idle_connections: state.idle_connections,
            max_size: self.pool.max_size(),
        }
    }
}

#[derive(Debug)]
pub struct PoolStatus {
    pub connections: u32,
    pub idle_connections: u32,
    pub max_size: u32,
}
