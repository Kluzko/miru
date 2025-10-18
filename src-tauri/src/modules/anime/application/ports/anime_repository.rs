use async_trait::async_trait;
use uuid::Uuid;

use crate::modules::anime::domain::{AnimeAggregate, AnimeDetailed, AnimeRelation};
use crate::shared::{
    application::pagination::{PaginatedResult, PaginationParams},
    domain::value_objects::AnimeProvider,
    errors::AppResult,
};

/// Port (interface) for anime repository following Hexagonal Architecture
/// This is a domain/application layer interface - infrastructure provides the implementation
#[async_trait]
pub trait AnimeRepository: Send + Sync {
    /// Save a new anime aggregate
    async fn save(&self, aggregate: &AnimeAggregate) -> AppResult<()>;

    /// Update an existing anime aggregate
    async fn update(&self, aggregate: &AnimeAggregate) -> AppResult<()>;

    /// Find anime by ID
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<AnimeDetailed>>;

    /// Find anime by external provider ID
    async fn find_by_external_id(
        &self,
        provider: AnimeProvider,
        external_id: &str,
    ) -> AppResult<Option<AnimeDetailed>>;

    /// Check if anime exists
    async fn exists(&self, id: Uuid) -> AppResult<bool>;

    /// Delete anime by ID
    async fn delete(&self, id: Uuid) -> AppResult<()>;

    /// Search anime with pagination
    async fn search(
        &self,
        query: &str,
        pagination: PaginationParams,
    ) -> AppResult<PaginatedResult<AnimeDetailed>>;

    /// Get all anime with pagination
    async fn find_all(
        &self,
        pagination: PaginationParams,
    ) -> AppResult<PaginatedResult<AnimeDetailed>>;
}

/// Port for anime relations repository (separated concern)
#[async_trait]
pub trait AnimeRelationsRepository: Send + Sync {
    /// Save relations for an anime
    async fn save_relations(&self, anime_id: Uuid, relations: Vec<AnimeRelation>) -> AppResult<()>;

    /// Get all relations for an anime
    async fn find_relations(&self, anime_id: Uuid) -> AppResult<Vec<AnimeRelation>>;

    /// Get bidirectional relations (both directions)
    async fn find_bidirectional_relations(&self, anime_id: Uuid) -> AppResult<Vec<AnimeRelation>>;

    /// Delete all relations for an anime
    async fn delete_relations(&self, anime_id: Uuid) -> AppResult<()>;
}

/// Port for complex anime queries (Specification Pattern)
#[async_trait]
pub trait AnimeQueryRepository: Send + Sync {
    /// Find anime by multiple criteria
    async fn find_by_criteria(
        &self,
        specification: AnimeSearchSpecification,
        pagination: PaginationParams,
    ) -> AppResult<PaginatedResult<AnimeDetailed>>;

    /// Count anime matching criteria
    async fn count_by_criteria(&self, specification: AnimeSearchSpecification) -> AppResult<u64>;
}

/// Specification for complex anime searches
#[derive(Debug, Clone, Default)]
pub struct AnimeSearchSpecification {
    pub title_contains: Option<String>,
    pub min_score: Option<f32>,
    pub max_score: Option<f32>,
    pub providers: Option<Vec<AnimeProvider>>,
    pub genres: Option<Vec<String>>,
    pub year: Option<i32>,
    pub status: Option<String>,
}
