use super::application::service::AnimeService;
use super::domain::entities::anime_detailed::AnimeDetailed;
use crate::modules::provider::AnimeProvider;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchAnimeRequest {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAnimeByIdRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateAnimeRequest {
    pub anime: AnimeDetailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeleteAnimeRequest {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetTopAnimeRequest {
    pub page: i32,
    pub limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetSeasonalAnimeRequest {
    pub year: i32,
    pub season: String,
    pub page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RecalculateScoresRequest {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetRecommendationsRequest {
    pub anime_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchAnimeExternalRequest {
    pub query: String,
    #[specta(type = Option<u32>)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAnimeByExternalIdRequest {
    pub id: String,
    pub preferred_provider: Option<AnimeProvider>,
}

#[tauri::command]
#[specta::specta]
pub async fn search_anime(
    request: SearchAnimeRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Vec<AnimeDetailed>, String> {
    anime_service
        .search_anime(&request.query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_anime_by_id(
    request: GetAnimeByIdRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Option<AnimeDetailed>, String> {
    log::info!("Getting anime by ID: '{}'", request.id);

    // Parse the string ID as UUID with proper error handling
    let anime_id = Uuid::parse_str(&request.id).map_err(|e| {
        log::error!("Invalid anime ID format '{}': {}", request.id, e);
        format!(
            "Invalid anime ID format '{}': {}. Expected a valid UUID.",
            request.id, e
        )
    })?;

    log::info!("Parsed UUID successfully: {}", anime_id);

    match anime_service.get_anime_by_id(&anime_id).await {
        Ok(Some(anime)) => {
            log::info!("Found anime: '{}' (ID: {})", anime.title.main, anime_id);
            Ok(Some(anime))
        }
        Ok(None) => {
            log::warn!("No anime found with ID: {}", anime_id);
            Ok(None)
        }
        Err(e) => {
            log::error!("Database error getting anime {}: {}", anime_id, e);
            Err(format!("Database error: {}", e))
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_top_anime(
    request: GetTopAnimeRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Vec<AnimeDetailed>, String> {
    anime_service
        .get_top_anime(request.limit as usize)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_seasonal_anime(
    request: GetSeasonalAnimeRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Vec<AnimeDetailed>, String> {
    anime_service
        .get_seasonal_anime(request.year, &request.season, request.page as usize)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn search_anime_external(
    request: SearchAnimeExternalRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Vec<AnimeDetailed>, String> {
    let limit = request.limit.unwrap_or(20).min(50); // Default 20, max 50

    anime_service
        .search_anime_external_only(&request.query, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_anime_by_external_id(
    request: GetAnimeByExternalIdRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Option<AnimeDetailed>, String> {
    anime_service
        .get_anime_by_external_id(&request.id, request.preferred_provider)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportRelationsRequest {
    pub anime_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportRelationsResponse {
    pub success: bool,
    pub relations_imported: usize,
    pub franchise_size: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetRelationsRequest {
    pub anime_id: Uuid,
}

#[tauri::command]
#[specta::specta]
pub async fn get_anime_relations(
    _request: GetRelationsRequest,
    _anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Vec<AnimeDetailed>, String> {
    // Legacy command - replaced by progressive relations API
    // Use get_basic_relations, get_detailed_relations, or discover_franchise instead
    Ok(Vec::new())
}

// ================================================================================================
// RELATIONS TAB COMMANDS (New lazy-loading franchise discovery)
// ================================================================================================

// Legacy relations commands removed - functionality moved to progressive_relations

// ================================================================================================
// ENRICHMENT COMMANDS (Automatic provider data enhancement)
// ================================================================================================

// Re-export auto-enrichment commands for automatic background enrichment
pub mod auto_enrichment;
pub use auto_enrichment::*;

// ================================================================================================
// PROGRESSIVE RELATIONS COMMANDS (Stage-based loading for better UX)
// ================================================================================================

// Re-export progressive relations commands for stage-based loading
pub mod progressive_relations;
pub use progressive_relations::*;
