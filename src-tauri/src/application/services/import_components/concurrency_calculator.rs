use crate::log_info;

/// Calculates optimal database concurrency limits based on system resources
pub struct ConcurrencyCalculator;

impl ConcurrencyCalculator {
    /// Calculate optimal concurrency for database operations based on system resources (reused from existing)
    pub fn calculate_db_concurrency() -> usize {
        const MIN_DB_CONCURRENCY: usize = 2;
        const MAX_DB_CONCURRENCY: usize = 20;
        const DB_CONCURRENCY_PER_CPU: usize = 2;

        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let optimal = (cpu_count * DB_CONCURRENCY_PER_CPU)
            .max(MIN_DB_CONCURRENCY)
            .min(MAX_DB_CONCURRENCY);

        log_info!(
            "Calculated DB concurrency: {} (CPUs: {}, multiplier: {}x)",
            optimal,
            cpu_count,
            DB_CONCURRENCY_PER_CPU
        );

        optimal
    }
}
