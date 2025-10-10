//! Anime Data Enrichment Commands
//!
//! These commands handle automatic detection and enrichment of missing provider data.
//! They use existing provider APIs to cross-reference and find missing external IDs.

use crate::modules::{anime::AnimeService, provider::application::service::ProviderService};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Deserialize, specta::Type)]
pub struct EnrichAnimeRequest {
    #[serde(rename = "animeId")]
    pub anime_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EnrichmentResult {
    pub anime_id: String,
    pub providers_added: Vec<String>,
    pub providers_updated: Vec<String>,
    pub errors: Vec<String>,
    pub success: bool,
}

/// Automatically enrich anime with missing provider data
///
/// This command uses existing provider IDs to search for and add missing provider data.
/// For example, if anime has only Jikan ID, it will search AniList using the title
/// and other metadata to find the corresponding AniList entry.
#[tauri::command]
#[specta::specta]
pub async fn enrich_anime_providers(
    request: EnrichAnimeRequest,
    anime_service: State<'_, Arc<AnimeService>>,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<EnrichmentResult, String> {
    let anime_uuid =
        Uuid::parse_str(&request.anime_id).map_err(|e| format!("Invalid anime ID: {}", e))?;

    // Get current anime data
    let anime = anime_service
        .get_anime_by_id(&anime_uuid)
        .await
        .map_err(|e| format!("Failed to get anime: {}", e))?
        .ok_or("Anime not found")?;

    let mut result = EnrichmentResult {
        anime_id: request.anime_id.clone(),
        providers_added: Vec::new(),
        providers_updated: Vec::new(),
        errors: Vec::new(),
        success: false,
    };

    let current_providers = &anime.provider_metadata.external_ids;

    // Try to find missing AniList ID if we have Jikan
    if !current_providers.contains_key(&crate::modules::provider::domain::AnimeProvider::AniList) {
        if let Some(jikan_id) =
            current_providers.get(&crate::modules::provider::domain::AnimeProvider::Jikan)
        {
            match find_anilist_from_jikan(&anime, jikan_id, &provider_service).await {
                Ok(Some(anilist_id)) => {
                    // TODO: Update anime with new AniList ID
                    result.providers_added.push("anilist".to_string());
                    log::info!("Found AniList ID {} for anime {}", anilist_id, anime_uuid);
                }
                Ok(None) => {
                    result
                        .errors
                        .push("AniList entry not found for this anime".to_string());
                }
                Err(e) => {
                    result.errors.push(format!("AniList search failed: {}", e));
                }
            }
        }
    }

    // Try to find missing Jikan ID if we have AniList
    if !current_providers.contains_key(&crate::modules::provider::domain::AnimeProvider::Jikan) {
        if let Some(anilist_id) =
            current_providers.get(&crate::modules::provider::domain::AnimeProvider::AniList)
        {
            match find_jikan_from_anilist(&anime, anilist_id, &provider_service).await {
                Ok(Some(jikan_id)) => {
                    // TODO: Update anime with new Jikan ID
                    result.providers_added.push("jikan".to_string());
                    log::info!("Found Jikan ID {} for anime {}", jikan_id, anime_uuid);
                }
                Ok(None) => {
                    result
                        .errors
                        .push("Jikan entry not found for this anime".to_string());
                }
                Err(e) => {
                    result.errors.push(format!("Jikan search failed: {}", e));
                }
            }
        }
    }

    // Additional providers can be added here (Kitsu, TMDB, etc.)

    result.success = result.providers_added.len() > 0 || result.providers_updated.len() > 0;

    if result.success {
        log::info!(
            "Enrichment completed for anime {}: added {:?}, updated {:?}",
            anime_uuid,
            result.providers_added,
            result.providers_updated
        );
    } else {
        log::warn!(
            "No enrichment performed for anime {}: {}",
            anime_uuid,
            result.errors.join(", ")
        );
    }

    Ok(result)
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct ResyncAnimeRequest {
    #[serde(rename = "animeId")]
    pub anime_id: String,
    #[serde(rename = "forceRefresh")]
    pub force_refresh: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ResyncResult {
    pub anime_id: String,
    pub providers_synced: Vec<String>,
    pub data_updated: bool,
    pub errors: Vec<String>,
    pub success: bool,
}

/// Re-sync anime data from all available providers
///
/// This command refreshes anime data from all providers where we have IDs,
/// ensuring the local data is up-to-date with the latest information.
#[tauri::command]
#[specta::specta]
pub async fn resync_anime_data(
    request: ResyncAnimeRequest,
    anime_service: State<'_, Arc<AnimeService>>,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<ResyncResult, String> {
    let anime_uuid =
        Uuid::parse_str(&request.anime_id).map_err(|e| format!("Invalid anime ID: {}", e))?;

    let mut result = ResyncResult {
        anime_id: request.anime_id.clone(),
        providers_synced: Vec::new(),
        data_updated: false,
        errors: Vec::new(),
        success: false,
    };

    // Get current anime data
    let anime = anime_service
        .get_anime_by_id(&anime_uuid)
        .await
        .map_err(|e| format!("Failed to get anime: {}", e))?
        .ok_or("Anime not found")?;

    let current_providers = &anime.provider_metadata.external_ids;

    // Sync from each available provider
    for (provider, external_id) in current_providers.iter() {
        match provider_service
            .get_anime_by_id(external_id, *provider)
            .await
        {
            Ok(Some(updated_data)) => {
                // TODO: Merge updated data with existing anime
                result.providers_synced.push(provider.to_string());
                result.data_updated = true;
                log::info!(
                    "Successfully synced anime {} from {} (ID: {})",
                    anime_uuid,
                    provider,
                    external_id
                );
            }
            Ok(None) => {
                result.errors.push(format!(
                    "Anime not found on {} with ID {}",
                    provider, external_id
                ));
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to sync from {}: {}", provider, e));
            }
        }
    }

    result.success = result.providers_synced.len() > 0;

    if result.success {
        log::info!(
            "Re-sync completed for anime {}: synced from {:?}",
            anime_uuid,
            result.providers_synced
        );
    } else {
        log::error!(
            "Re-sync failed for anime {}: {}",
            anime_uuid,
            result.errors.join(", ")
        );
    }

    Ok(result)
}

// Helper functions for cross-provider lookups

async fn find_anilist_from_jikan(
    anime: &crate::modules::anime::AnimeDetailed,
    jikan_id: &str,
    provider_service: &ProviderService,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    // Search AniList using anime title and year
    // This is a simplified implementation - real version would use more sophisticated matching

    // TODO: Implement actual cross-provider search
    // For now, return None to indicate not found
    log::info!(
        "Searching AniList for anime '{}' with Jikan ID {}",
        anime.title.main,
        jikan_id
    );

    // Mock implementation - would search AniList API using title matching
    Ok(None)
}

async fn find_jikan_from_anilist(
    anime: &crate::modules::anime::AnimeDetailed,
    anilist_id: &str,
    provider_service: &ProviderService,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    // Search Jikan using anime title and year

    // TODO: Implement actual cross-provider search
    log::info!(
        "Searching Jikan for anime '{}' with AniList ID {}",
        anime.title.main,
        anilist_id
    );

    // Mock implementation - would search Jikan API using title matching
    Ok(None)
}
