pub mod anime_service;
pub mod collection_service;
pub mod import_components;
pub mod import_service;
pub mod provider_manager;

pub use anime_service::AnimeService;
pub use collection_service::CollectionService;
pub use import_service::ImportService;
pub use provider_manager::ProviderManager;

// Re-export import types for external use
pub use import_components::{
    ExistingAnime, ImportError, ImportResult, ImportedAnime, SkippedAnime, ValidatedAnime,
    ValidationResult,
};
