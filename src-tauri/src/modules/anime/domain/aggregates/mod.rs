/// Aggregates for Anime Bounded Context
///
/// Following DDD Aggregate pattern:
/// - Each aggregate has a root entity
/// - All access to child entities goes through the root
/// - Aggregates maintain consistency boundaries
/// - Aggregates publish domain events for state changes
pub mod anime_aggregate;

pub use anime_aggregate::*;
