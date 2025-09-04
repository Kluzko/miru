use crate::domain::entities::{Anime, Collection, CollectionAnime};
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait CollectionRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Collection>>;
    async fn find_by_name(&self, name: &str) -> AppResult<Option<Collection>>;
    async fn get_all(&self) -> AppResult<Vec<Collection>>;
    async fn save(&self, collection: &Collection) -> AppResult<Collection>;
    async fn update(&self, collection: &Collection) -> AppResult<Collection>;
    async fn delete(&self, id: &Uuid) -> AppResult<()>;

    // Collection-Anime relationship methods
    async fn add_anime_to_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
        user_score: Option<f32>,
        notes: Option<String>,
    ) -> AppResult<()>;

    async fn remove_anime_from_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
    ) -> AppResult<()>;

    async fn get_collection_anime(&self, collection_id: &Uuid) -> AppResult<Vec<Anime>>;

    async fn get_collection_entry(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
    ) -> AppResult<Option<CollectionAnime>>;

    async fn update_collection_entry(&self, entry: &CollectionAnime) -> AppResult<()>;
}
