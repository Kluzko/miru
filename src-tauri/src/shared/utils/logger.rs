use log::{debug, error, info};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the logging system
/// This should be called once at application startup
pub fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info) // Default level
            .filter_module("miru", log::LevelFilter::Debug) // More verbose for our app
            .filter_module("diesel", log::LevelFilter::Warn) // Reduce diesel noise
            .filter_module("reqwest", log::LevelFilter::Warn) // Reduce HTTP noise
            .filter_module("tokio", log::LevelFilter::Warn) // Reduce tokio noise
            .format_timestamp_secs()
            .format_target(false)
            .format_module_path(false)
            .init();

        info!("Logging system initialized");
    });
}

/// Macro for structured logging with context
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        log::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        log::error!($($arg)*)
    };
}

/// Structured logging helpers for common patterns
pub struct LogContext;

impl LogContext {
    /// Log database operations
    pub fn db_operation(operation: &str, table: &str, duration_ms: Option<u64>) {
        match duration_ms {
            Some(duration) => info!("DB: {} on {} completed in {}ms", operation, table, duration),
            None => debug!("DB: Starting {} on {}", operation, table),
        }
    }

    /// Log API calls
    pub fn api_call(provider: &str, endpoint: &str, status: &str, duration_ms: Option<u64>) {
        match duration_ms {
            Some(duration) => info!(
                "API: {} {} {} in {}ms",
                provider, endpoint, status, duration
            ),
            None => debug!("API: Starting {} {}", provider, endpoint),
        }
    }

    /// Log import operations
    pub fn import_progress(current: usize, total: usize, title: &str) {
        info!("Import: [{}/{}] Processing '{}'", current, total, title);
    }

    /// Log search operations
    pub fn search_operation(query: &str, provider: Option<&str>, results: Option<usize>) {
        match (provider, results) {
            (Some(p), Some(r)) => info!("Search: '{}' via {} returned {} results", query, p, r),
            (Some(p), None) => debug!("Search: Starting '{}' via {}", query, p),
            (None, Some(r)) => info!("Search: '{}' returned {} results", query, r),
            (None, None) => debug!("Search: Starting '{}'", query),
        }
    }

    /// Log errors with context
    pub fn error_with_context(error: &dyn std::error::Error, context: &str) {
        error!("{}: {}", context, error);
    }

    /// Log performance metrics
    pub fn performance_metric(operation: &str, duration_ms: u64, additional_info: Option<&str>) {
        match additional_info {
            Some(info) => info!(
                "Performance: {} took {}ms ({})",
                operation, duration_ms, info
            ),
            None => info!("Performance: {} took {}ms", operation, duration_ms),
        }
    }
}

/// Helper for timing operations
pub struct TimedOperation {
    start: std::time::Instant,
    operation: String,
}

impl TimedOperation {
    pub fn new(operation: &str) -> Self {
        debug!("Starting: {}", operation);
        Self {
            start: std::time::Instant::now(),
            operation: operation.to_string(),
        }
    }

    pub fn finish(self) -> u64 {
        let duration = self.start.elapsed().as_millis() as u64;
        LogContext::performance_metric(&self.operation, duration, None);
        duration
    }

    pub fn finish_with_info(self, info: &str) -> u64 {
        let duration = self.start.elapsed().as_millis() as u64;
        LogContext::performance_metric(&self.operation, duration, Some(info));
        duration
    }
}
