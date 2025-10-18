/// Anime Aggregate Root
///
/// Following DDD Aggregate pattern:
/// - AnimeAggregate is the root entity
/// - All business operations go through the aggregate root
/// - Maintains invariants and consistency
/// - Publishes domain events for state changes
///
/// This aggregate encapsulates:
/// - Anime core data (title, score, metadata)
/// - Relations to other anime (child entities)
/// - Quality metrics and scoring
/// - Provider synchronization state
mod anime;
mod relations;

pub use anime::AnimeAggregate;
pub use relations::AnimeRelation;

// Re-export the existing AnimeDetailed for backward compatibility
// Eventually this will be fully replaced by AnimeAggregate
pub use crate::modules::anime::domain::entities::anime_detailed::AnimeDetailed;
