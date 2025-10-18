pub mod ingestion_service;
pub mod ports;
pub mod service;
pub mod use_cases;

// Re-export commonly used types
pub use ports::{
    AnimeQueryRepository, AnimeRelationsRepository, AnimeRepository, AnimeSearchSpecification,
    EventPublisher, ProviderClient,
};

pub use use_cases::{
    CreateAnimeCommand, CreateAnimeHandler, CreateAnimeResult, DiscoverRelationsCommand,
    DiscoverRelationsHandler, DiscoverRelationsResult, SearchAnimeHandler, SearchAnimeQuery,
    SearchAnimeResult, UpdateAnimeScoreCommand, UpdateAnimeScoreHandler, UpdateAnimeScoreResult,
};
