/// Isolated test database utility that creates and cleans up temporary databases
/// Each test gets its own database that is automatically dropped when the test completes
///
/// This uses TEST_DATABASE_URL from the environment to connect to the test database server.
use diesel::r2d2::{self, ConnectionManager};
use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use futures::future::BoxFuture;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
static TEST_DB_COUNTER: AtomicU32 = AtomicU32::new(0);

pub type TestPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Isolated test database that automatically cleans up on drop
///
/// # Example
/// ```rust
/// #[tokio::test]
/// async fn test_something() {
///     let test_db = TestDb::new();
///     test_db.run_test(|pool| {
///         Box::pin(async move {
///             // Your test code here using the pool
///             let services = build_test_services_with_pool(pool);
///             // Test...
///         })
///     }).await;
///     // Database automatically dropped here
/// }
/// ```
pub struct TestDb {
    default_db_url: String,
    name: String,
    pool: TestPool,
}


impl TestDb {
    /// Creates a new isolated test database with a unique name
    ///
    /// Database name format: test_db_{process_id}_{counter}
    /// This ensures multiple test processes can run in parallel
    ///
    /// Uses TEST_DATABASE_URL environment variable to connect to the test database server
    pub fn new() -> Self {
        dotenvy::dotenv().ok();

        // Generate unique database name
        let name = format!(
            "test_db_{}_{}",
            std::process::id(),
            TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst)
        );

        // Connect to test database server (using TEST_DATABASE_URL)
        let test_db_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set in .env for tests");

        let mut conn = PgConnection::establish(&test_db_url)
            .expect("Failed to connect to test database server");

        // Create isolated test database
        sql_query(format!("CREATE DATABASE {}", name))
            .execute(&mut conn)
            .expect(&format!("Failed to create test database: {}", name));

        // Build connection string for the new isolated test database
        // Replace the database name in the URL (everything after the last '/')
        let isolated_db_url = if let Some(last_slash) = test_db_url.rfind('/') {
            format!("{}/{}", &test_db_url[..last_slash], name)
        } else {
            panic!("Invalid TEST_DATABASE_URL format: {}", test_db_url);
        };

        // Create connection pool for the isolated test database
        let manager = ConnectionManager::<PgConnection>::new(isolated_db_url);
        let pool = r2d2::Pool::builder()
            .max_size(5) // Smaller pool for test databases
            .test_on_check_out(true)
            .build(manager)
            .expect("Failed to build test database connection pool");

        Self {
            default_db_url: test_db_url,
            name,
            pool,
        }
    }

    /// Run a test with this isolated database
    ///
    /// Automatically runs migrations before executing the test
    /// The test receives a connection pool to use
    pub async fn run_test(&self, test: impl Fn(TestPool) -> BoxFuture<'static, ()>) {
        let conn = &mut self
            .pool
            .get()
            .expect("Unable to connect to the test database");

        // Run migrations on test database
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Unable to migrate the test database");

        // Execute test with the pool
        test(self.pool.clone()).await;
    }

    /// Get the connection pool for this test database
    ///
    /// Use this when you need direct access to the pool
    /// without using run_test()
    pub fn pool(&self) -> TestPool {
        self.pool.clone()
    }

    /// Get the database name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for TestDb {
    /// Automatically clean up test database when TestDb is dropped
    ///
    /// - Terminates all connections to the database
    /// - Drops the database
    /// - If thread is panicking, database is left for debugging
    fn drop(&mut self) {
        if thread::panicking() {
            eprintln!(
                "⚠️  TestDb leaking database '{}' due to panic - database preserved for debugging",
                self.name
            );
            return;
        }

        let mut conn = PgConnection::establish(&self.default_db_url)
            .expect("Failed to connect to test database server for cleanup");

        // Terminate all connections to test database
        let terminate_result = sql_query(format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
            self.name
        ))
        .execute(&mut conn);

        if let Err(e) = terminate_result {
            eprintln!(
                "⚠️  Failed to terminate connections for '{}': {}",
                self.name, e
            );
        }

        // Drop test database
        let drop_result =
            sql_query(format!("DROP DATABASE IF EXISTS {}", self.name)).execute(&mut conn);

        match drop_result {
            Ok(_) => {
                log::debug!("✓ Cleaned up test database: {}", self.name);
            }
            Err(e) => {
                eprintln!("⚠️  Failed to drop test database '{}': {}", self.name, e);
            }
        }
    }
}

// Tests for TestDb are in the e2e tests themselves
// They verify that:
// 1. Database is created
// 2. Migrations run successfully
// 3. Database is automatically cleaned up on drop
