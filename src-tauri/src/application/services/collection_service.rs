use crate::domain::{
    entities::{Anime, Collection, CollectionAnime},
    events::{AnimeEvent, CollectionEvent},
    repositories::{AnimeRepository, CollectionRepository},
};
use crate::infrastructure::cache::RedisCache;
use crate::shared::errors::{AppError, AppResult};
use std::sync::Arc;
use uuid::Uuid;

pub struct CollectionService {
    collection_repo: Arc<dyn CollectionRepository>,
    anime_repo: Arc<dyn AnimeRepository>,
    cache: Arc<RedisCache>,
}

impl CollectionService {
    pub fn new(
        collection_repo: Arc<dyn CollectionRepository>,
        anime_repo: Arc<dyn AnimeRepository>,
        cache: Arc<RedisCache>,
    ) -> Self {
        Self {
            collection_repo,
            anime_repo,
            cache,
        }
    }

    pub async fn create_collection(
        &self,
        name: String,
        description: Option<String>,
    ) -> AppResult<Collection> {
        // Validate name
        crate::shared::utils::Validator::validate_collection_name(&name)?;

        // Check if collection with same name exists
        if let Some(_) = self.collection_repo.find_by_name(&name).await? {
            return Err(AppError::ValidationError(format!(
                "Collection with name '{}' already exists",
                name
            )));
        }

        // Create collection
        let mut collection = Collection::new(name.clone());
        if let Some(desc) = description {
            collection = collection.with_description(desc);
        }

        // Save to database
        let saved = self.collection_repo.save(&collection).await?;

        // Invalidate collections cache
        let _ = self.cache.delete("collections:all").await;

        Ok(saved)
    }

    pub async fn get_collection(&self, id: &Uuid) -> AppResult<Option<Collection>> {
        // Check cache
        let cache_key = format!("collection:{}", id);
        if let Ok(Some(cached)) = self.cache.get::<Collection>(&cache_key).await {
            return Ok(Some(cached));
        }

        // Get from database
        let collection = self.collection_repo.find_by_id(id).await?;

        if let Some(ref coll) = collection {
            // Cache the result
            let _ = self.cache.set(&cache_key, coll, 3600).await;
        }

        Ok(collection)
    }

    pub async fn get_all_collections(&self) -> AppResult<Vec<Collection>> {
        // Check cache
        let cache_key = "collections:all";
        if let Ok(Some(cached)) = self.cache.get::<Vec<Collection>>(cache_key).await {
            return Ok(cached);
        }

        // Get from database
        let collections = self.collection_repo.get_all().await?;

        // Cache the results
        let _ = self.cache.set(cache_key, &collections, 1800).await;

        Ok(collections)
    }

    pub async fn update_collection(
        &self,
        id: &Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> AppResult<Collection> {
        // Get existing collection
        let mut collection =
            self.collection_repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Collection with ID {} not found", id))
            })?;

        // Update fields
        if let Some(new_name) = name {
            crate::shared::utils::Validator::validate_collection_name(&new_name)?;

            // Check if another collection has this name
            if let Some(existing) = self.collection_repo.find_by_name(&new_name).await? {
                if existing.id != collection.id {
                    return Err(AppError::ValidationError(format!(
                        "Collection with name '{}' already exists",
                        new_name
                    )));
                }
            }

            collection.rename(new_name);
        }

        if description.is_some() {
            collection.update_description(description);
        }

        // Save to database
        let updated = self.collection_repo.update(&collection).await?;

        // Invalidate cache
        let cache_key = format!("collection:{}", id);
        let _ = self.cache.delete(&cache_key).await;
        let _ = self.cache.delete("collections:all").await;

        Ok(updated)
    }

    pub async fn delete_collection(&self, id: &Uuid) -> AppResult<()> {
        // Check if collection exists
        let collection =
            self.collection_repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Collection with ID {} not found", id))
            })?;

        // Delete from database
        self.collection_repo.delete(id).await?;

        // Invalidate cache
        let cache_key = format!("collection:{}", id);
        let _ = self.cache.delete(&cache_key).await;
        let _ = self.cache.delete("collections:all").await;

        // Invalidate anime in collection cache
        let anime_cache_key = format!("collection:{}:anime", id);
        let _ = self.cache.delete(&anime_cache_key).await;

        Ok(())
    }

    pub async fn add_anime_to_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
        user_score: Option<f32>,
        notes: Option<String>,
    ) -> AppResult<()> {
        // Validate score if provided
        if let Some(score) = user_score {
            crate::shared::utils::Validator::validate_score(score)?;
        }

        // Check if collection exists
        let mut collection = self
            .collection_repo
            .find_by_id(collection_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Collection with ID {} not found", collection_id))
            })?;

        // Check if anime exists
        let anime =
            self.anime_repo.find_by_id(anime_id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Anime with ID {} not found", anime_id))
            })?;

        // Check if already in collection
        if collection.contains_anime(anime_id) {
            return Err(AppError::ValidationError(
                "Anime already exists in this collection".to_string(),
            ));
        }

        // Add to collection
        self.collection_repo
            .add_anime_to_collection(collection_id, anime_id, user_score, notes)
            .await?;

        // Update collection anime_ids
        collection.add_anime(*anime_id);
        let _ = self.collection_repo.update(&collection).await;

        // Invalidate cache
        self.invalidate_collection_cache(collection_id).await?;

        Ok(())
    }

    pub async fn remove_anime_from_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
    ) -> AppResult<()> {
        // Check if collection exists
        let mut collection = self
            .collection_repo
            .find_by_id(collection_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Collection with ID {} not found", collection_id))
            })?;

        // Check if anime is in collection
        if !collection.contains_anime(anime_id) {
            return Err(AppError::ValidationError(
                "Anime not found in this collection".to_string(),
            ));
        }

        // Remove from collection
        self.collection_repo
            .remove_anime_from_collection(collection_id, anime_id)
            .await?;

        // Update collection anime_ids
        collection.remove_anime(anime_id);
        let _ = self.collection_repo.update(&collection).await;

        // Invalidate cache
        self.invalidate_collection_cache(collection_id).await?;

        Ok(())
    }

    pub async fn get_collection_anime(&self, collection_id: &Uuid) -> AppResult<Vec<Anime>> {
        // Check cache
        let cache_key = format!("collection:{}:anime", collection_id);
        if let Ok(Some(cached)) = self.cache.get::<Vec<Anime>>(&cache_key).await {
            return Ok(cached);
        }

        // Check if collection exists
        let _ = self
            .collection_repo
            .find_by_id(collection_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Collection with ID {} not found", collection_id))
            })?;

        // Get anime from database
        let anime_list = self
            .collection_repo
            .get_collection_anime(collection_id)
            .await?;

        // Cache the results
        let _ = self.cache.set(&cache_key, &anime_list, 3600).await;

        Ok(anime_list)
    }

    pub async fn update_anime_in_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
        user_score: Option<f32>,
        notes: Option<String>,
    ) -> AppResult<()> {
        // Validate score if provided
        if let Some(score) = user_score {
            crate::shared::utils::Validator::validate_score(score)?;
        }

        // Get existing entry
        let mut entry = self
            .collection_repo
            .get_collection_entry(collection_id, anime_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Anime not found in collection".to_string()))?;

        // Update entry
        entry.update_score(user_score);
        entry.update_notes(notes);

        // Save to database
        self.collection_repo.update_collection_entry(&entry).await?;

        // Invalidate cache
        self.invalidate_collection_cache(collection_id).await?;

        Ok(())
    }

    pub async fn get_collection_statistics(
        &self,
        collection_id: &Uuid,
    ) -> AppResult<CollectionStats> {
        // Get collection
        let collection = self
            .collection_repo
            .find_by_id(collection_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Collection with ID {} not found", collection_id))
            })?;

        // Get all anime in collection
        let anime_list = self.get_collection_anime(collection_id).await?;

        // Calculate statistics
        let total_anime = anime_list.len();
        let total_episodes: i32 = anime_list.iter().filter_map(|a| a.episodes).sum();

        let avg_score = if total_anime > 0 {
            anime_list.iter().map(|a| a.composite_score).sum::<f32>() / total_anime as f32
        } else {
            0.0
        };

        let mut genre_distribution = std::collections::HashMap::new();
        for anime in &anime_list {
            for genre in &anime.genres {
                *genre_distribution.entry(genre.name.clone()).or_insert(0) += 1;
            }
        }

        let mut status_distribution = std::collections::HashMap::new();
        for anime in &anime_list {
            *status_distribution
                .entry(anime.status.to_string())
                .or_insert(0) += 1;
        }

        Ok(CollectionStats {
            total_anime,
            total_episodes,
            average_score: avg_score,
            genre_distribution,
            status_distribution,
            top_rated: anime_list
                .iter()
                .filter(|a| a.composite_score >= 8.0)
                .cloned()
                .collect(),
        })
    }

    async fn invalidate_collection_cache(&self, collection_id: &Uuid) -> AppResult<()> {
        let cache_keys = vec![
            format!("collection:{}", collection_id),
            format!("collection:{}:anime", collection_id),
            "collections:all".to_string(),
        ];

        for key in cache_keys {
            let _ = self.cache.delete(&key).await;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionStats {
    pub total_anime: usize,
    pub total_episodes: i32,
    pub average_score: f32,
    pub genre_distribution: std::collections::HashMap<String, i32>,
    pub status_distribution: std::collections::HashMap<String, i32>,
    pub top_rated: Vec<Anime>,
}
