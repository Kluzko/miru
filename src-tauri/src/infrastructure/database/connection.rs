use crate::shared::errors::AppError;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool};
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new() -> Result<Self, AppError> {
        let database_url = env::var("DATABASE_URL")?;

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .max_size(15)
            .min_idle(Some(5))
            .connection_timeout(std::time::Duration::from_secs(30))
            .build(manager)
            .map_err(|e| AppError::DatabaseError(format!("Failed to create pool: {}", e)))?;

        Ok(Self { pool })
    }

    pub fn get_connection(&self) -> Result<DbConnection, AppError> {
        self.pool.get().map_err(AppError::from)
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}
