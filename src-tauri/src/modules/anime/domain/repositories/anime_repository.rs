use super::super::entities::anime_detailed::AnimeDetailed;
use crate::shared::domain::value_objects::AnimeProvider;
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
// JsonValue import removed - no longer needed with simplified relations approach
use uuid::Uuid;

/// Anime with relation metadata for batch fetching
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnimeWithRelationMetadata {
    pub anime: AnimeDetailed,
    pub relation_type: String,
    pub synced_at: DateTime<Utc>,
}

#[async_trait]
pub trait AnimeRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<AnimeDetailed>>;
    async fn find_by_external_id(
        &self,
        provider: &AnimeProvider,
        external_id: &str,
    ) -> AppResult<Option<AnimeDetailed>>;
    async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<AnimeDetailed>>;
    async fn save(&self, anime: &AnimeDetailed) -> AppResult<AnimeDetailed>;
    /// Batch save operation for bulk imports
    #[allow(dead_code)]
    async fn save_batch(&self, anime_list: &[AnimeDetailed]) -> AppResult<Vec<AnimeDetailed>>;
    /// Update existing anime data
    async fn update(&self, anime: &AnimeDetailed) -> AppResult<AnimeDetailed>;
    /// Delete anime from repository
    async fn delete(&self, id: &Uuid) -> AppResult<()>;
    /// Get all anime with pagination
    #[allow(dead_code)]
    async fn get_all(&self, offset: i64, limit: i64) -> AppResult<Vec<AnimeDetailed>>;
    async fn find_by_title_variations(
        &self,
        search_title: &str,
    ) -> AppResult<Option<AnimeDetailed>>;

    // Relations management
    async fn get_relations(&self, anime_id: &Uuid) -> AppResult<Vec<(Uuid, String)>>;
    async fn save_relations(&self, anime_id: &Uuid, relations: &[(Uuid, String)]) -> AppResult<()>;

    // Batch fetch anime with their relation metadata
    async fn get_anime_with_relations(
        &self,
        anime_id: &Uuid,
    ) -> AppResult<Vec<AnimeWithRelationMetadata>>;
    // Note: enrich_relation method removed - with simplified approach,
    // all enrichment is done by updating the complete anime record directly
}
