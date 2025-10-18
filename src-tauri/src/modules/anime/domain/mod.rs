// Domain layer - Following DDD + Clean Architecture patterns

pub mod aggregates; // NEW - DDD Aggregate roots
pub mod entities;
pub mod events; // NEW - Domain events
pub mod repositories;
pub mod services;
pub mod traits;
pub mod value_objects;

// Re-exports for easy access
pub use aggregates::anime_aggregate::{AnimeAggregate, AnimeRelation}; // NEW - Aggregate root and child entity
pub use entities::anime_detailed::AnimeDetailed; // Existing (will migrate to aggregate)
pub use events::*; // NEW - Domain events
pub use repositories::anime_repository::AnimeRepository;
