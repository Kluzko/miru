use crate::modules::provider::{AnimeProvider, ProviderService};
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
                total_requests: snapshot.total_requests as u32,
                consecutive_failures: snapshot.consecutive_failures,
                avg_response_time_ms: snapshot.avg_response_time.map(|d| d.as_millis() as u32),
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
    pub total_requests: u32,
    pub consecutive_failures: u32,
    pub avg_response_time_ms: Option<u32>,
}

// Enhanced provider management types
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderConfig {
    pub provider: AnimeProvider,
    pub enabled: bool,
    pub priority: u8,
    pub rate_limit: u32,
    pub timeout_ms: u32,
    pub retry_attempts: u32,
    pub cache_duration_secs: u32,
    pub api_key: Option<String>,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateProviderConfigRequest {
    pub provider: AnimeProvider,
    pub config: ProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CacheStatistics {
    pub provider: AnimeProvider,
    pub hit_rate: f64,
    pub total_entries: u32,
    pub cache_size_bytes: u32,
    pub last_cleared: Option<String>,
}

// Enhanced provider management commands

#[tauri::command]
#[specta::specta]
pub async fn get_provider_config(
    provider: AnimeProvider,
    _provider_service: State<'_, Arc<ProviderService>>,
) -> Result<ProviderConfig, String> {
    // TODO: Get actual config from provider service
    Ok(ProviderConfig {
        provider: provider.clone(),
        enabled: true,
        priority: 1,
        rate_limit: 60,
        timeout_ms: 5000,
        retry_attempts: 3,
        cache_duration_secs: 3600,
        api_key: None,
        base_url: match provider {
            AnimeProvider::AniList => "https://graphql.anilist.co".to_string(),
            AnimeProvider::Jikan => "https://api.jikan.moe/v4".to_string(),
            AnimeProvider::Kitsu => "https://kitsu.io/api/edge".to_string(),
            AnimeProvider::TMDB => "https://api.themoviedb.org/3".to_string(),
            AnimeProvider::AniDB => "https://api.anidb.net:9001".to_string(),
        },
    })
}

#[tauri::command]
#[specta::specta]
pub async fn update_provider_config(
    request: UpdateProviderConfigRequest,
    _provider_service: State<'_, Arc<ProviderService>>,
) -> Result<(), String> {
    // TODO: Update actual provider config
    println!("Updating config for provider: {:?}", request.provider);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_all_provider_configs(
    _provider_service: State<'_, Arc<ProviderService>>,
) -> Result<Vec<ProviderConfig>, String> {
    // TODO: Get actual configs from all providers
    Ok(vec![
        ProviderConfig {
            provider: AnimeProvider::AniList,
            enabled: true,
            priority: 1,
            rate_limit: 90,
            timeout_ms: 5000,
            retry_attempts: 3,
            cache_duration_secs: 3600,
            api_key: None,
            base_url: "https://graphql.anilist.co".to_string(),
        },
        ProviderConfig {
            provider: AnimeProvider::Jikan,
            enabled: true,
            priority: 2,
            rate_limit: 60,
            timeout_ms: 10000,
            retry_attempts: 2,
            cache_duration_secs: 7200,
            api_key: None,
            base_url: "https://api.jikan.moe/v4".to_string(),
        },
    ])
}

#[tauri::command]
#[specta::specta]
pub async fn get_cache_statistics(
    _provider_service: State<'_, Arc<ProviderService>>,
) -> Result<Vec<CacheStatistics>, String> {
    // TODO: Get actual cache stats from provider service
    Ok(vec![
        CacheStatistics {
            provider: AnimeProvider::AniList,
            hit_rate: 87.3,
            total_entries: 2456,
            cache_size_bytes: 47513600, // ~45.2 MB
            last_cleared: Some("2023-01-01T00:00:00Z".to_string()),
        },
        CacheStatistics {
            provider: AnimeProvider::Jikan,
            hit_rate: 75.4,
            total_entries: 1245,
            cache_size_bytes: 29491200, // ~28.1 MB
            last_cleared: Some("2023-01-01T12:00:00Z".to_string()),
        },
    ])
}

#[tauri::command]
#[specta::specta]
pub async fn clear_provider_cache(
    provider: AnimeProvider,
    _provider_service: State<'_, Arc<ProviderService>>,
) -> Result<(), String> {
    // TODO: Clear actual provider cache
    println!("Clearing cache for provider: {:?}", provider);
    Ok(())
}
