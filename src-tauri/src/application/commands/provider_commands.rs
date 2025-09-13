use crate::application::services::provider_manager::ProviderManager;
use crate::domain::traits::anime_provider_client::RateLimiterInfo;
use crate::domain::value_objects::{AgeRestrictionInfo, AnimeProvider, UnifiedAgeRestriction};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SetPrimaryProviderRequest {
    pub provider: AnimeProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetProviderRateLimitRequest {
    pub provider: AnimeProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderStatus {
    pub provider: AnimeProvider,
    pub is_primary: bool,
    pub enabled: bool,
    pub rate_limit_info: Option<RateLimiterInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProvidersListResponse {
    pub providers: Vec<ProviderStatus>,
    pub primary_provider: AnimeProvider,
}

/// Get list of all providers with their status
#[tauri::command]
#[specta::specta]
pub async fn list_providers(
    provider_manager: State<'_, Arc<Mutex<ProviderManager>>>,
) -> Result<ProvidersListResponse, String> {
    let manager = provider_manager.lock().await;
    let primary_provider = manager.get_primary_provider();
    let enabled_providers = manager.get_enabled_providers();

    let providers = vec![AnimeProvider::Jikan, AnimeProvider::AniList]
        .into_iter()
        .map(|provider| {
            let rate_limit_info = manager.get_provider_rate_limit(&provider);
            ProviderStatus {
                provider: provider.clone(),
                is_primary: provider == primary_provider,
                enabled: enabled_providers.contains(&provider),
                rate_limit_info,
            }
        })
        .collect();

    Ok(ProvidersListResponse {
        providers,
        primary_provider,
    })
}

/// Set the primary provider
#[tauri::command]
#[specta::specta]
pub async fn set_primary_provider(
    request: SetPrimaryProviderRequest,
    provider_manager: State<'_, Arc<Mutex<ProviderManager>>>,
) -> Result<(), String> {
    let mut manager = provider_manager.lock().await;
    manager
        .set_primary_provider(request.provider)
        .map_err(|e| e.to_string())
}

/// Get the current primary provider
#[tauri::command]
#[specta::specta]
pub async fn get_primary_provider(
    provider_manager: State<'_, Arc<Mutex<ProviderManager>>>,
) -> Result<AnimeProvider, String> {
    let manager = provider_manager.lock().await;
    Ok(manager.get_primary_provider())
}

/// Get enabled providers
#[tauri::command]
#[specta::specta]
pub async fn get_enabled_providers(
    provider_manager: State<'_, Arc<Mutex<ProviderManager>>>,
) -> Result<Vec<AnimeProvider>, String> {
    let manager = provider_manager.lock().await;
    Ok(manager.get_enabled_providers())
}

/// Get rate limit info for a specific provider
#[tauri::command]
#[specta::specta]
pub async fn get_provider_rate_limit(
    request: GetProviderRateLimitRequest,
    provider_manager: State<'_, Arc<Mutex<ProviderManager>>>,
) -> Result<Option<RateLimiterInfo>, String> {
    let manager = provider_manager.lock().await;
    Ok(manager.get_provider_rate_limit(&request.provider))
}

/// Get all age restrictions with display names for frontend
#[tauri::command]
#[specta::specta]
pub async fn get_age_restrictions() -> Result<Vec<AgeRestrictionInfo>, String> {
    Ok(UnifiedAgeRestriction::all_with_info())
}
