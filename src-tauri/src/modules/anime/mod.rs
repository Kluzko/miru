pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

// Re-exports for easy external access
pub use application::ingestion_service::{
    AnimeIngestionService, AnimeSource, IngestionOptions, IngestionResult, JobPriority,
};
pub use application::service::AnimeService;
pub use domain::{AnimeDetailed, AnimeRepository};

// Relationship entities removed - using simplified approach with AnimeWithRelationMetadata

// Re-export common value objects for shorter imports
pub use domain::value_objects::{AnimeStatus, AnimeTier, AnimeType};

// Re-export infrastructure components
