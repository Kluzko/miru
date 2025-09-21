use crate::modules::provider::ProviderManager;
use crate::shared::errors::AppResult;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderInfo {
    pub name: String,
    pub enabled: bool,
    pub is_primary: bool,
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AgeRestrictionInfo {
    pub provider: String,
    pub max_age: Option<u8>,
    pub content_rating: String,
}

#[tauri::command]
#[specta::specta]
pub async fn list_providers(
    provider_manager: State<'_, Arc<Mutex<ProviderManager>>>,
) -> Result<Vec<ProviderInfo>, String> {
    let manager = provider_manager.lock().await;

    let providers = vec![
        ProviderInfo {
            name: "Jikan".to_string(),
            enabled: true,
            is_primary: manager.get_primary_provider().to_string() == "Jikan",
            rate_limit_per_minute: 60,
        },
        ProviderInfo {
            name: "AniList".to_string(),
            enabled: true,
            is_primary: manager.get_primary_provider().to_string() == "AniList",
            rate_limit_per_minute: 90,
        },
    ];

    Ok(providers)
}

#[tauri::command]
#[specta::specta]
pub async fn set_primary_provider(
    provider_name: String,
    provider_manager: State<'_, Arc<Mutex<ProviderManager>>>,
) -> Result<(), String> {
    let mut manager = provider_manager.lock().await;

    let provider = match provider_name.as_str() {
        "Jikan" => crate::modules::provider::AnimeProvider::Jikan,
        "AniList" => crate::modules::provider::AnimeProvider::AniList,
        _ => return Err("Unknown provider".to_string()),
    };

    manager.set_primary_provider(provider);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_primary_provider(
    provider_manager: State<'_, Arc<Mutex<ProviderManager>>>,
) -> Result<String, String> {
    let manager = provider_manager.lock().await;
    Ok(manager.get_primary_provider().to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_enabled_providers(
    provider_manager: State<'_, Arc<Mutex<ProviderManager>>>,
) -> Result<Vec<String>, String> {
    Ok(vec!["Jikan".to_string(), "AniList".to_string()])
}

#[tauri::command]
#[specta::specta]
pub async fn get_provider_rate_limit(provider_name: String) -> Result<u32, String> {
    match provider_name.as_str() {
        "Jikan" => Ok(60),
        "AniList" => Ok(90),
        _ => Err("Unknown provider".to_string()),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_age_restrictions() -> Result<Vec<AgeRestrictionInfo>, String> {
    Ok(vec![
        AgeRestrictionInfo {
            provider: "Jikan".to_string(),
            max_age: Some(18),
            content_rating: "R - 17+ (violence & profanity)".to_string(),
        },
        AgeRestrictionInfo {
            provider: "AniList".to_string(),
            max_age: Some(18),
            content_rating: "Mature".to_string(),
        },
    ])
}
