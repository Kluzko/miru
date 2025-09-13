use crate::domain::entities::AnimeDetailed;
use crate::domain::value_objects::AnimeProvider;
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use uuid::Uuid;

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
}
