use super::application::service::CollectionService;
use super::domain::entities::collection::Collection;
use crate::modules::anime::AnimeDetailed;

use crate::{log_debug, log_error, log_info};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCollectionRequest {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateCollectionRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeleteCollectionRequest {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AddAnimeToCollectionRequest {
    pub collection_id: Uuid,
    pub anime_id: Uuid,
    pub user_score: Option<f32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RemoveAnimeFromCollectionRequest {
    pub collection_id: Uuid,
    pub anime_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCollectionAnimeRequest {
    pub collection_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateAnimeInCollectionRequest {
    pub collection_id: Uuid,
    pub anime_id: Uuid,
    pub user_score: Option<f32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCollectionStatisticsRequest {
    pub collection_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportAnimeBatchRequest {
    pub titles: Vec<String>,
    pub collection_id: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub async fn create_collection(
    request: CreateCollectionRequest,
    collection_service: State<'_, Arc<CollectionService>>,
) -> Result<Collection, String> {
    collection_service
        .create_collection(request.name, request.description)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_collection(
    request: GetCollectionRequest,
    collection_service: State<'_, Arc<CollectionService>>,
) -> Result<Option<Collection>, String> {
    collection_service
        .get_collection(&request.id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_all_collections(
    collection_service: State<'_, Arc<CollectionService>>,
) -> Result<Vec<Collection>, String> {
    collection_service
        .get_all_collections()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn update_collection(
    request: UpdateCollectionRequest,
    collection_service: State<'_, Arc<CollectionService>>,
) -> Result<Collection, String> {
    collection_service
        .update_collection(&request.id, request.name, request.description)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_collection(
    request: DeleteCollectionRequest,
    collection_service: State<'_, Arc<CollectionService>>,
) -> Result<(), String> {
    collection_service
        .delete_collection(&request.id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn add_anime_to_collection(
    request: AddAnimeToCollectionRequest,
    collection_service: State<'_, Arc<CollectionService>>,
) -> Result<(), String> {
    log_debug!(
        "add_anime_to_collection command called - collection: {}, anime: {}",
        request.collection_id,
        request.anime_id
    );

    let result = collection_service
        .add_anime_to_collection(
            &request.collection_id,
            &request.anime_id,
            request.user_score,
            request.notes,
        )
        .await
        .map_err(|e| e.to_string());

    match &result {
        Ok(_) => {
            log_info!(
                "Successfully added anime {} to collection {}",
                request.anime_id,
                request.collection_id
            );
        }
        Err(e) => {
            log_error!("Failed to add anime to collection: {}", e);
        }
    }

    result
}

#[tauri::command]
#[specta::specta]
pub async fn remove_anime_from_collection(
    request: RemoveAnimeFromCollectionRequest,
    collection_service: State<'_, Arc<CollectionService>>,
) -> Result<(), String> {
    collection_service
        .remove_anime_from_collection(&request.collection_id, &request.anime_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_collection_anime(
    request: GetCollectionAnimeRequest,
    collection_service: State<'_, Arc<CollectionService>>,
) -> Result<Vec<AnimeDetailed>, String> {
    collection_service
        .get_collection_anime(&request.collection_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn update_anime_in_collection(
    request: UpdateAnimeInCollectionRequest,
    collection_service: State<'_, Arc<CollectionService>>,
) -> Result<(), String> {
    collection_service
        .update_anime_in_collection(
            &request.collection_id,
            &request.anime_id,
            request.user_score,
            request.notes,
        )
        .await
        .map_err(|e| e.to_string())
}
