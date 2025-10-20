// Clean architecture modules
pub mod adapters;
pub mod decorators;
pub mod http_client;
pub mod monitoring;

// Re-export commonly used types
pub use adapters::ProviderRepositoryAdapter;
pub use decorators::CachingRepositoryDecorator;
pub use http_client::{RateLimitClient, RetryPolicy};
pub use monitoring::{HealthMonitor, MetricsCollector};
