//! Progressive Relations Commands
//!
//! This module provides the main command for fetching anime relations.
//! Auto-discovery is handled by the backend service transparently.

use crate::modules::anime::domain::{
    repositories::anime_repository::AnimeWithRelationMetadata,
    services::anime_relations_service::AnimeRelationsService,
};
use std::sync::Arc;
use tauri::State;

/// Get anime with relations using optimized batch approach
///
/// This command fetches complete anime data with relation metadata in a single call.
/// If no relations exist in the database, the backend automatically discovers and saves them
/// from AniList before returning the results.
///
/// Returns a vector of related anime with their relation types and sync timestamps.
#[tauri::command]
#[specta::specta]
pub async fn get_anime_with_relations(
    anime_id: String,
    relations_service: State<'_, Arc<AnimeRelationsService>>,
) -> Result<Vec<AnimeWithRelationMetadata>, String> {
    log::debug!(
        "Command: get_anime_with_relations for anime_id: {}",
        anime_id
    );

    relations_service
        .get_anime_with_relations(&anime_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get anime with relations for {}: {}", anime_id, e);
            format!("Failed to load relations: {}", e)
        })
}
