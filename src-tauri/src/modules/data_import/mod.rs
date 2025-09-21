pub mod application;
pub mod commands;
pub mod domain;

// Re-exports for easy external access
pub use application::service::ImportService;
pub use domain::{ImportCoordinator, ImportExecutor, ValidationService};

// Re-export common types for shorter imports
pub use domain::services::import_components::types::{
    ExistingAnime, ImportError, ImportResult, ImportedAnime, SkippedAnime, ValidatedAnime,
    ValidationResult,
};
