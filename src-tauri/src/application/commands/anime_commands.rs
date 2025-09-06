use crate::application::services::anime_service::AnimeService;
use crate::domain::entities::Anime;
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
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAnimeByMalIdRequest {
    pub mal_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateAnimeRequest {
    pub anime: Anime,
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

#[tauri::command]
#[specta::specta]
pub async fn search_anime(
    request: SearchAnimeRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Vec<Anime>, String> {
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
) -> Result<Option<Anime>, String> {
    anime_service
        .get_anime_by_id(&request.id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_top_anime(
    request: GetTopAnimeRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Vec<Anime>, String> {
    anime_service
        .get_top_anime(request.page, request.limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_seasonal_anime(
    request: GetSeasonalAnimeRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Vec<Anime>, String> {
    anime_service
        .get_seasonal_anime(request.year, &request.season, request.page)
        .await
        .map_err(|e| e.to_string())
}
