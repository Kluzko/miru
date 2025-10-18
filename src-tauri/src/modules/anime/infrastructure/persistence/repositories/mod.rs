pub mod anime_query_repository_impl;
pub mod anime_relations_repository_impl;
/// Repository implementations for anime persistence following DDD principles
///
/// This module contains three separate repository implementations:
///
/// 1. **AnimeRepositoryImpl** - Core CRUD operations for anime entities
///    - Basic operations: find, save, update, delete
///    - Batch operations for performance
///    - Manages anime base data, genres, studios, quality metrics, and external IDs
///
/// 2. **AnimeRelationsRepositoryImpl** - Manages anime relationships
///    - Save/load relations between anime
///    - Bidirectional relation support
///    - Inverse relation type calculation
///
/// 3. **AnimeQueryRepositoryImpl** - Complex queries and search operations
///    - Specification pattern for complex queries
///    - Advanced search with fuzzy matching
///    - Genre/studio-based searches
///    - Top-rated and recently updated queries
pub mod anime_repository_impl;

// Re-export the implementations for convenience
pub use anime_query_repository_impl::{AnimeQueryRepositoryImpl, AnimeSearchSpecification};
pub use anime_relations_repository_impl::{inverse_relation_type, AnimeRelationsRepositoryImpl};
pub use anime_repository_impl::AnimeRepositoryImpl;
