use crate::application::services::collection_service::CollectionService;
use crate::application::services::import_service::{
    ImportResult, ImportService, ValidatedAnime, ValidationResult,
};
use crate::domain::entities::{AnimeDetailed, Collection};
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportFromCsvRequest {
    pub csv_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ValidateAnimeTitlesRequest {
    pub titles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportValidatedAnimeRequest {
    pub validated_anime: Vec<ValidatedAnime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportSeasonalRequest {
    pub year: i32,
    pub season: String,
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
    println!(
        "DEBUG: add_anime_to_collection command called - collection: {}, anime: {}",
        request.collection_id, request.anime_id
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
            println!(
                "DEBUG: Successfully added anime {} to collection {}",
                request.anime_id, request.collection_id
            );
        }
        Err(e) => {
            println!("DEBUG: Failed to add anime to collection: {}", e);
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

#[tauri::command]
#[specta::specta]
pub async fn import_anime_batch(
    request: ImportAnimeBatchRequest,
    import_service: State<'_, Arc<ImportService>>,
    app_handle: tauri::AppHandle,
) -> Result<ImportResult, String> {
    import_service
        .import_anime_batch_with_progress(request.titles, Some(app_handle))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn import_from_csv(
    request: ImportFromCsvRequest,
    import_service: State<'_, Arc<ImportService>>,
    app_handle: tauri::AppHandle,
) -> Result<ImportResult, String> {
    // Parse CSV first to get titles
    let mut reader = csv::Reader::from_reader(request.csv_content.as_bytes());
    let mut titles = Vec::new();

    for result in reader.records() {
        match result {
            Ok(record) => {
                if let Some(first_field) = record.get(0) {
                    if !first_field.trim().is_empty() {
                        titles.push(first_field.trim().to_string());
                    }
                }
            }
            Err(_) => continue,
        }
    }

    if titles.is_empty() {
        return Err("No valid data found in CSV".to_string());
    }

    import_service
        .import_anime_batch_with_progress(titles, Some(app_handle))
        .await
        .map_err(|e| e.to_string())
}

// New two-phase import commands
#[tauri::command]
#[specta::specta]
pub async fn validate_anime_titles(
    request: ValidateAnimeTitlesRequest,
    import_service: State<'_, Arc<ImportService>>,
) -> Result<ValidationResult, String> {
    import_service
        .validate_anime_titles(request.titles)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn import_validated_anime(
    request: ImportValidatedAnimeRequest,
    import_service: State<'_, Arc<ImportService>>,
) -> Result<ImportResult, String> {
    println!(
        "DEBUG: import_validated_anime command called with {} anime",
        request.validated_anime.len()
    );
    let result = import_service
        .import_validated_anime(request.validated_anime)
        .await
        .map_err(|e| e.to_string());

    match &result {
        Ok(import_result) => {
            println!(
                "DEBUG: Import completed - imported: {}, failed: {}, skipped: {}",
                import_result.imported.len(),
                import_result.failed.len(),
                import_result.skipped.len()
            );
        }
        Err(e) => {
            println!("DEBUG: Import failed with error: {}", e);
        }
    }

    result
}
