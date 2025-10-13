/// Database test utilities with singleton pattern
///
/// Provides thread-safe access to test database with proper isolation
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool};
use std::sync::{Arc, Mutex, Once};

type PgPool = Pool<ConnectionManager<PgConnection>>;

static INIT: Once = Once::new();
static mut DB_POOL: Option<Arc<PgPool>> = None;

/// Get or create singleton database pool for tests
pub fn get_test_db_pool() -> Arc<PgPool> {
    unsafe {
        INIT.call_once(|| {
            dotenvy::dotenv().ok();
            let test_db_url = std::env::var("TEST_DATABASE_URL")
                .expect("TEST_DATABASE_URL must be set in .env for tests");

            let manager = ConnectionManager::<PgConnection>::new(test_db_url);
            let pool = r2d2::Pool::builder()
                .max_size(10)
                .build(manager)
                .expect("Failed to create test database pool");

            DB_POOL = Some(Arc::new(pool));
        });

        DB_POOL.as_ref().unwrap().clone()
    }
}

/// Clean all test tables - use at the start of each test
pub fn clean_test_db() {
    let pool = get_test_db_pool();
    let mut conn = pool.get().expect("Failed to get DB connection");

    diesel::sql_query("TRUNCATE TABLE background_jobs RESTART IDENTITY CASCADE")
        .execute(&mut conn)
        .expect("Failed to clean background_jobs");

    diesel::sql_query("TRUNCATE TABLE anime RESTART IDENTITY CASCADE")
        .execute(&mut conn)
        .expect("Failed to clean anime");

    diesel::sql_query("TRUNCATE TABLE anime_relations RESTART IDENTITY CASCADE")
        .execute(&mut conn)
        .expect("Failed to clean anime_relations");

    diesel::sql_query("TRUNCATE TABLE collections RESTART IDENTITY CASCADE")
        .execute(&mut conn)
        .ok(); // May not exist in all schemas
}

/// Global test mutex for serialization
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Acquire test lock to ensure tests run serially
/// Returns a guard that releases the lock when dropped
pub fn acquire_test_lock() -> std::sync::MutexGuard<'static, ()> {
    // Handle poisoned mutex by recovering from panic
    match TEST_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
