pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

// Re-exports for easy external access
pub use application::ingestion_service::{
    AnimeIngestionService, AnimeSource, IngestionOptions, IngestionResult, JobPriority,
};
pub use application::service::AnimeService;
pub use domain::{AnimeAggregate, AnimeDetailed, AnimeRepository};

// Relationship entities removed - using simplified approach with AnimeWithRelationMetadata

// Re-export common value objects for shorter imports
pub use domain::value_objects::{AnimeStatus, AnimeTier, AnimeType};

// Re-export infrastructure components

// Re-export application layer use cases and ports
pub use application::{
    AnimeQueryRepository, AnimeRelationsRepository, AnimeRepository as IAnimeRepository,
    AnimeSearchSpecification, CreateAnimeCommand, CreateAnimeHandler, CreateAnimeResult,
    DiscoverRelationsCommand, DiscoverRelationsHandler, DiscoverRelationsResult, EventPublisher,
    ProviderClient, SearchAnimeHandler, SearchAnimeQuery, SearchAnimeResult,
    UpdateAnimeScoreCommand, UpdateAnimeScoreHandler, UpdateAnimeScoreResult,
};
