use super::super::domain::{
    entities::collection::Collection, repositories::collection_repository::CollectionRepository,
};
use crate::modules::anime::domain::{
    entities::anime_detailed::AnimeDetailed, repositories::anime_repository::AnimeRepository,
};
use crate::shared::errors::{AppError, AppResult};
// use crate::shared::utils::logger::{LogContext, TimedOperation};
use crate::{log_debug, log_info};
use std::sync::Arc;
use uuid::Uuid;

pub struct CollectionService {
    collection_repo: Arc<dyn CollectionRepository>,
    anime_repo: Arc<dyn AnimeRepository>,
}

impl CollectionService {
    pub fn new(
        collection_repo: Arc<dyn CollectionRepository>,
        anime_repo: Arc<dyn AnimeRepository>,
    ) -> Self {
        Self {
            collection_repo,
            anime_repo,
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
        let mut collection = Collection::new(name);
        if let Some(desc) = description {
            collection = collection.with_description(desc);
        }

        // Save to database
        let saved = self.collection_repo.save(&collection).await?;

        Ok(saved)
    }

    pub async fn get_collection(&self, id: &Uuid) -> AppResult<Option<Collection>> {
        // Get from database
        let collection = self.collection_repo.find_by_id(id).await?;

        Ok(collection)
    }

    pub async fn get_all_collections(&self) -> AppResult<Vec<Collection>> {
        // Get from database
        let collections = self.collection_repo.get_all().await?;

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

        Ok(updated)
    }

    pub async fn delete_collection(&self, id: &Uuid) -> AppResult<()> {
        // Check if collection exists
        let _collection =
            self.collection_repo.find_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("Collection with ID {} not found", id))
            })?;

        // Delete from database
        self.collection_repo.delete(id).await?;

        Ok(())
    }

    pub async fn add_anime_to_collection(
        &self,
        collection_id: &Uuid,
        anime_id: &Uuid,
        user_score: Option<f32>,
        notes: Option<String>,
    ) -> AppResult<()> {
        log_debug!(
            "Adding anime {} to collection {} (score: {:?}, notes: {:?})",
            anime_id,
            collection_id,
            user_score,
            notes
        );

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

        self.anime_repo
            .find_by_id(anime_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Anime with ID {} not found", anime_id)))?;

        // Check if already in collection
        if collection.contains_anime(anime_id) {
            return Err(AppError::ValidationError(
                "Anime already exists in this collection".to_string(),
            ));
        }

        // Add to collection
        log_debug!("Calling repository to add anime to collection");
        self.collection_repo
            .add_anime_to_collection(collection_id, anime_id, user_score, notes)
            .await?;

        // Update collection anime_ids
        log_debug!("Updating collection anime_ids list");
        collection.add_anime(*anime_id);
        let _ = self.collection_repo.update(&collection).await;

        log_info!(
            "Successfully added anime {} to collection {}",
            anime_id,
            collection_id
        );
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

        Ok(())
    }

    pub async fn get_collection_anime(
        &self,
        collection_id: &Uuid,
    ) -> AppResult<Vec<AnimeDetailed>> {
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
    pub top_rated: Vec<AnimeDetailed>,
}
