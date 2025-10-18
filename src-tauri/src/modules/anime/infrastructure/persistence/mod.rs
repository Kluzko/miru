// Modular repository implementations following DDD
pub mod repositories;

// Shared mapping utilities
pub mod mapper;

// Re-export repository implementations
pub use repositories::{
    inverse_relation_type, AnimeQueryRepositoryImpl, AnimeRelationsRepositoryImpl,
    AnimeRepositoryImpl, AnimeSearchSpecification,
};
