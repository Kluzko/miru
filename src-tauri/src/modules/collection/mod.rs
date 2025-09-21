pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

// Re-exports for easy external access
pub use application::service::CollectionService;
pub use domain::{Collection, CollectionAnime, CollectionRepository};
