//! Provider-related Tauri commands orchestrated through ProviderService
//!
//! IMPORTANT: All relationship/franchise discovery commands use AniList exclusively
//! due to superior GraphQL API performance (1 call vs 13+ calls for other providers).
//!
//! This module follows DDD principles by:
//! - All operations orchestrated through ProviderService (Application Layer)
//! - No direct adapter access from commands (maintains separation of concerns)
//! - Proper error handling and result mapping
//! - Clean command interfaces for frontend consumption

use crate::modules::provider::{
    application::service::{ProviderService, RelationshipCapabilities},
    infrastructure::adapters::anilist::models::{CategorizedFranchise, FranchiseRelation},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// Simple anime relation for basic relationship queries
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AnimeRelation {
    pub id: u32,
    pub relation_type: String,
}

/// Get basic franchise relations (AniList exclusive - optimized GraphQL)
///
/// This command provides basic relationship discovery using AniList's superior GraphQL API.
/// Performance: ~0.4-1.0 seconds vs 10+ seconds with other providers.
#[tauri::command]
#[specta::specta]
pub async fn get_franchise_relations(
    anime_id: u32,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<Vec<AnimeRelation>, String> {
    let relations = provider_service
        .get_anime_relations(anime_id)
        .await
        .map_err(|e| e.to_string())?;

    let anime_relations = relations
        .into_iter()
        .map(|(id, relation_type)| AnimeRelation { id, relation_type })
        .collect();

    Ok(anime_relations)
}

/// Discover complete franchise with detailed metadata (AniList exclusive)
///
/// Returns comprehensive franchise data including titles, years, episodes, and formats.
/// Uses single GraphQL call for entire franchise discovery - vastly superior to REST APIs.
/// Performance: Complete franchise (74+ items) discovered in <1 second.
#[tauri::command]
#[specta::specta]
pub async fn discover_franchise_details(
    anime_id: u32,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<Vec<FranchiseRelation>, String> {
    provider_service
        .discover_franchise_details(anime_id)
        .await
        .map_err(|e| e.to_string())
}

/// Discover and categorize complete franchise (AniList exclusive - RECOMMENDED)
///
/// The most comprehensive franchise discovery method available.
/// Returns franchise content intelligently categorized into:
/// - Main Story: Core seasons and sequels in chronological order
/// - Side Stories: Spin-offs and alternate storylines
/// - Movies: Theatrical releases
/// - OVAs/Specials: Additional content and extras
/// - Other: Miscellaneous relations
///
/// Performance: Complete categorized franchise in 0.4-2.2 seconds
#[tauri::command]
#[specta::specta]
pub async fn discover_categorized_franchise(
    anime_id: u32,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<CategorizedFranchise, String> {
    provider_service
        .discover_categorized_franchise(anime_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get relationship discovery capabilities and performance information
///
/// Returns information about why relationship discovery is AniList-exclusive
/// and performance comparisons with other providers.
#[tauri::command]
#[specta::specta]
pub async fn get_relationship_capabilities(
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<RelationshipCapabilities, String> {
    Ok(provider_service.get_relationship_capabilities())
}
