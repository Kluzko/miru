pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod traits;

// Re-exports for easy external access
pub use domain::{AnimeProvider, ProviderMetadata};
pub use infrastructure::{
    cache::provider_cache::ProviderCache, manager::provider_manager::ProviderManager,
};
