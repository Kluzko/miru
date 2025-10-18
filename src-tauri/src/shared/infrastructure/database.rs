use crate::log_info;
use crate::shared::errors::AppError;
use crate::shared::utils::logger::LogContext;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};
use std::env;
use std::time::Duration;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Debug)]
pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new() -> Result<Self, AppError> {
        let database_url = Self::get_validated_database_url()?;

        // Enhanced connection pool configuration with dynamic sizing
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        let pool_config = Self::get_optimal_pool_config();
        let pool = r2d2::Pool::builder()
            // Dynamic pool sizing based on system resources
            .max_size(pool_config.max_size)
            .min_idle(Some(pool_config.min_idle))
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
            pool_config.min_idle
        );

        Ok(Self { pool })
    }

    /// Create a Database instance from an existing pool (useful for testing)
    pub fn from_pool(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Validate and retrieve database URL with security measures
    fn get_validated_database_url() -> Result<String, AppError> {
        let database_url = env::var("DATABASE_URL").map_err(|_| {
            AppError::DatabaseError("DATABASE_URL environment variable not found".to_string())
        })?;

        // Validate URL format and basic security checks
        if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
            return Err(AppError::DatabaseError(
                "Invalid database URL format. Must start with postgres:// or postgresql://"
                    .to_string(),
            ));
        }

        // Ensure URL doesn't contain obvious security issues
        if database_url.contains("password=") || database_url.len() < 20 {
            return Err(AppError::DatabaseError(
                "Database URL appears to be malformed or insecure".to_string(),
            ));
        }

        // Log connection attempt without exposing credentials
        log_info!(
            "Initializing database connection to: {}",
            database_url.split('@').last().unwrap_or("unknown_host")
        );

        Ok(database_url)
    }

    /// Calculate optimal pool configuration based on system resources
    fn get_optimal_pool_config() -> PoolConfig {
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        // Base pool size on CPU count but cap for desktop app
        let max_size = std::cmp::min(cpu_count * 2, 20);
        let min_idle = std::cmp::max(2, max_size / 4);

        PoolConfig {
            max_size: max_size as u32,
            min_idle: min_idle as u32,
        }
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

    /// Get the underlying connection pool (useful for testing and repository initialization)
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

#[derive(Debug)]
pub struct PoolStatus {
    pub connections: u32,
    pub idle_connections: u32,
    pub max_size: u32,
}

#[derive(Debug)]
struct PoolConfig {
    max_size: u32,
    min_idle: u32,
}
