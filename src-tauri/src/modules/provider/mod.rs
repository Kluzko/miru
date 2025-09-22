pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod traits;

// Re-exports for easy external access - only export what's actually used
pub use domain::{AnimeProvider, ProviderFactoryManager, ProviderMetadata};
pub use infrastructure::service::ProviderService;

// Legacy exports for backward compatibility
pub use infrastructure::cache::ProviderCache;
