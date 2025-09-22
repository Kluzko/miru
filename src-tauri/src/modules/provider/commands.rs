use crate::modules::{
    anime::AnimeDetailed,
    provider::{AnimeProvider, ProviderService},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderInfo {
    pub provider: AnimeProvider,
    pub name: String,
    pub enabled: bool,
    pub is_primary: bool,
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AgeRestrictionInfo {
    pub provider: AnimeProvider,
    pub max_age: Option<u8>,
    pub content_rating: String,
}

#[tauri::command]
#[specta::specta]
pub async fn list_providers(
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<Vec<ProviderInfo>, String> {
    let primary_provider = provider_service.get_primary_provider().await;

    let providers = vec![
        ProviderInfo {
            provider: AnimeProvider::Jikan,
            name: "MyAnimeList (Jikan)".to_string(),
            enabled: true,
            is_primary: primary_provider == AnimeProvider::Jikan,
            rate_limit_per_minute: 60,
        },
        ProviderInfo {
            provider: AnimeProvider::AniList,
            name: "AniList".to_string(),
            enabled: true,
            is_primary: primary_provider == AnimeProvider::AniList,
            rate_limit_per_minute: 30,
        },
    ];

    Ok(providers)
}

#[tauri::command]
#[specta::specta]
pub async fn set_primary_provider(
    provider: AnimeProvider,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<(), String> {
    provider_service
        .set_primary_provider(provider)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_primary_provider(
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<AnimeProvider, String> {
    Ok(provider_service.get_primary_provider().await)
}

#[tauri::command]
#[specta::specta]
pub async fn get_enabled_providers(
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<Vec<AnimeProvider>, String> {
    Ok(provider_service.get_enabled_providers().await)
}

#[tauri::command]
#[specta::specta]
pub async fn get_provider_rate_limit(
    provider: AnimeProvider,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<u32, String> {
    if let Some(rate_limit_info) = provider_service.get_provider_rate_limit(&provider) {
        Ok(rate_limit_info.requests_per_minute)
    } else {
        Err("Rate limit info not available".to_string())
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_age_restrictions() -> Result<Vec<AgeRestrictionInfo>, String> {
    Ok(vec![
        AgeRestrictionInfo {
            provider: AnimeProvider::Jikan,
            max_age: Some(18),
            content_rating: "R - 17+ (violence & profanity)".to_string(),
        },
        AgeRestrictionInfo {
            provider: AnimeProvider::AniList,
            max_age: Some(18),
            content_rating: "Mature".to_string(),
        },
    ])
}

#[tauri::command]
#[specta::specta]
#[allow(dead_code)] // May be used by frontend
pub async fn search_anime_comprehensive(
    query: String,
    limit: Option<usize>,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<Vec<AnimeDetailed>, String> {
    let limit = limit.unwrap_or(20).min(50); // Default 20, max 50

    provider_service
        .search_anime_comprehensive(&query, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
#[allow(dead_code)] // May be used by frontend
pub async fn get_anime_by_id_comprehensive(
    id: String,
    preferred_provider: Option<AnimeProvider>,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<Option<AnimeDetailed>, String> {
    provider_service
        .get_anime_by_id_comprehensive(&id, preferred_provider)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
#[allow(dead_code)] // May be used by frontend
pub async fn get_provider_health_status(
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<Vec<ProviderHealthInfo>, String> {
    let mut health_info = Vec::new();

    for provider in [AnimeProvider::Jikan, AnimeProvider::AniList] {
        if let Some(health) = provider_service.get_provider_health(&provider).await {
            let snapshot = health.snapshot();
            health_info.push(ProviderHealthInfo {
                provider: provider.clone(),
                is_healthy: snapshot.is_healthy,
                success_rate: (snapshot.successful_requests as f64
                    / snapshot.total_requests.max(1) as f64)
                    * 100.0,
                total_requests: snapshot.total_requests,
                consecutive_failures: snapshot.consecutive_failures,
                avg_response_time_ms: snapshot.avg_response_time.map(|d| d.as_millis() as u64),
            });
        }
    }

    Ok(health_info)
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderHealthInfo {
    pub provider: AnimeProvider,
    pub is_healthy: bool,
    pub success_rate: f64,
    pub total_requests: u64,
    pub consecutive_failures: u32,
    pub avg_response_time_ms: Option<u64>,
}
