/// Domain Events for Anime Bounded Context
///
/// Following Event Sourcing and DDD patterns, these events represent
/// state changes in the anime aggregate that other parts of the system
/// may be interested in.
mod anime_events;

pub use anime_events::*;
