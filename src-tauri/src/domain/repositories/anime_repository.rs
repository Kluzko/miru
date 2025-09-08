use crate::domain::entities::Anime;
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AnimeRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Anime>>;
    async fn find_by_mal_id(&self, mal_id: i32) -> AppResult<Option<Anime>>;
    async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<Anime>>;
    async fn save(&self, anime: &Anime) -> AppResult<Anime>;
    async fn save_batch(&self, anime_list: &[Anime]) -> AppResult<Vec<Anime>>;
    async fn update(&self, anime: &Anime) -> AppResult<Anime>;
    async fn delete(&self, id: &Uuid) -> AppResult<()>;
    async fn get_all(&self, offset: i64, limit: i64) -> AppResult<Vec<Anime>>;
    async fn find_by_title_variations(&self, search_title: &str) -> AppResult<Option<Anime>>;
}
