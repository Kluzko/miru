//! Automatic Anime Enrichment on Loading
//!
//! This module handles automatic provider data enrichment when anime is loaded.
//! It runs silently in the background to enhance missing provider data.

use crate::modules::{anime::AnimeService, provider::application::service::ProviderService};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Deserialize, specta::Type)]
pub struct AutoEnrichRequest {
    #[serde(rename = "animeId")]
    pub anime_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AutoEnrichResult {
    pub anime_id: String,
    pub enrichment_performed: bool,
    pub providers_found: Vec<String>,
    pub should_reload: bool,
}

/// Automatically enrich anime data when loading anime details
///
/// This command runs automatically when an anime detail page loads.
/// It silently checks for missing provider data and enriches it in the background.
#[tauri::command]
#[specta::specta]
pub async fn auto_enrich_on_load(
    request: AutoEnrichRequest,
    anime_service: State<'_, Arc<AnimeService>>,
    provider_service: State<'_, Arc<ProviderService>>,
) -> Result<AutoEnrichResult, String> {
    let anime_uuid =
        Uuid::parse_str(&request.anime_id).map_err(|e| format!("Invalid anime ID: {}", e))?;

    let mut result = AutoEnrichResult {
        anime_id: request.anime_id.clone(),
        enrichment_performed: false,
        providers_found: Vec::new(),
        should_reload: false,
    };

    // Get current anime data
    let anime = match anime_service.get_anime_by_id(&anime_uuid).await {
        Ok(Some(anime)) => anime,
        Ok(None) => return Ok(result), // Anime not found, no enrichment needed
        Err(e) => {
            log::error!("Failed to get anime for auto-enrichment: {}", e);
            return Ok(result);
        }
    };

    let current_providers = &anime.provider_metadata.external_ids;

    // Check if critical providers are missing
    let has_anilist =
        current_providers.contains_key(&crate::modules::provider::domain::AnimeProvider::AniList);
    let has_jikan =
        current_providers.contains_key(&crate::modules::provider::domain::AnimeProvider::Jikan);

    // If both critical providers are present, no auto-enrichment needed
    if has_anilist && has_jikan {
        return Ok(result);
    }

    log::info!(
        "Auto-enriching anime '{}' (ID: {}) - Missing: AniList={}, Jikan={}",
        anime.title.main,
        anime_uuid,
        !has_anilist,
        !has_jikan
    );

    // Try to find missing AniList ID using Jikan
    if !has_anilist && has_jikan {
        if let Some(jikan_id) =
            current_providers.get(&crate::modules::provider::domain::AnimeProvider::Jikan)
        {
            match find_anilist_by_title(&anime, &provider_service).await {
                Ok(Some(anilist_id)) => {
                    // Get the AniList data to merge with existing data
                    match provider_service
                        .get_anime_by_id(
                            &anilist_id.to_string(),
                            crate::modules::provider::domain::AnimeProvider::AniList,
                        )
                        .await
                    {
                        Ok(Some(anilist_data)) => {
                            // Use the existing data quality service to merge the data intelligently
                            match merge_and_save_enriched_data(
                                &anime,
                                &anilist_data,
                                &anime_service,
                            )
                            .await
                            {
                                Ok(merged_anime) => {
                                    log::info!(
                                        "✅ Auto-enrichment: Successfully merged and saved AniList data for '{}' (Jikan {} -> AniList {})",
                                        merged_anime.title.main,
                                        jikan_id,
                                        anilist_id
                                    );
                                    result.enrichment_performed = true;
                                    result.providers_found.push("anilist".to_string());
                                    result.should_reload = true;
                                }
                                Err(e) => {
                                    log::error!("Failed to merge and save AniList data: {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            log::warn!("AniList ID {} not found in provider", anilist_id);
                        }
                        Err(e) => {
                            log::error!("Failed to get AniList data for ID {}: {}", anilist_id, e);
                        }
                    }
                }
                Ok(None) => {
                    log::info!(
                        "Auto-enrichment: No AniList match found for '{}'",
                        anime.title.main
                    );
                }
                Err(e) => {
                    log::warn!("Auto-enrichment AniList search failed: {}", e);
                }
            }
        }
    }

    // Try to find missing Jikan ID using AniList
    if !has_jikan && has_anilist {
        if let Some(anilist_id) =
            current_providers.get(&crate::modules::provider::domain::AnimeProvider::AniList)
        {
            match find_jikan_by_title(&anime, &provider_service).await {
                Ok(Some(jikan_id)) => {
                    // Get the Jikan data to merge with existing data
                    match provider_service
                        .get_anime_by_id(
                            &jikan_id.to_string(),
                            crate::modules::provider::domain::AnimeProvider::Jikan,
                        )
                        .await
                    {
                        Ok(Some(jikan_data)) => {
                            // Use the existing data quality service to merge the data intelligently
                            match merge_and_save_enriched_data(&anime, &jikan_data, &anime_service)
                                .await
                            {
                                Ok(merged_anime) => {
                                    log::info!(
                                        "✅ Auto-enrichment: Successfully merged and saved Jikan data for '{}' (AniList {} -> Jikan {})",
                                        merged_anime.title.main,
                                        anilist_id,
                                        jikan_id
                                    );
                                    result.enrichment_performed = true;
                                    result.providers_found.push("jikan".to_string());
                                    result.should_reload = true;
                                }
                                Err(e) => {
                                    log::error!("Failed to merge and save Jikan data: {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            log::warn!("Jikan ID {} not found in provider", jikan_id);
                        }
                        Err(e) => {
                            log::error!("Failed to get Jikan data for ID {}: {}", jikan_id, e);
                        }
                    }
                }
                Ok(None) => {
                    log::info!(
                        "Auto-enrichment: No Jikan match found for '{}'",
                        anime.title.main
                    );
                }
                Err(e) => {
                    log::warn!("Auto-enrichment Jikan search failed: {}", e);
                }
            }
        }
    }

    if result.enrichment_performed {
        log::info!(
            "Auto-enrichment completed for anime '{}': found {:?}",
            anime.title.main,
            result.providers_found
        );
    }

    Ok(result)
}

// Helper function to find AniList ID using existing Jikan data
async fn find_anilist_by_title(
    anime: &crate::modules::anime::AnimeDetailed,
    provider_service: &ProviderService,
) -> Result<Option<u32>, String> {
    use crate::modules::provider::domain::value_objects::provider_enum::AnimeProvider;

    // Get Jikan ID from current provider metadata
    let jikan_id = anime
        .provider_metadata
        .external_ids
        .get(&AnimeProvider::Jikan)
        .ok_or("No Jikan ID found for cross-provider search")?;

    log::info!(
        "Using Jikan ID {} to find AniList match for '{}'",
        jikan_id,
        anime.title.main
    );

    // Step 1: Get full anime data from Jikan to extract all title variants
    let jikan_data = match provider_service
        .get_anime_by_id(jikan_id, AnimeProvider::Jikan)
        .await
    {
        Ok(Some(data)) => data,
        Ok(None) => {
            log::warn!("Jikan ID {} not found, cannot cross-reference", jikan_id);
            return Ok(None);
        }
        Err(e) => {
            log::error!("Failed to get Jikan data for ID {}: {}", jikan_id, e);
            return Ok(None);
        }
    };

    // Step 2: Extract multiple search queries from Jikan data
    let mut search_queries = Vec::new();

    // Primary titles
    search_queries.push(jikan_data.title.main.clone());
    if let Some(english) = &jikan_data.title.english {
        if !english.is_empty() && english != &jikan_data.title.main {
            search_queries.push(english.clone());
        }
    }
    if let Some(romaji) = &jikan_data.title.romaji {
        if !romaji.is_empty() && romaji != &jikan_data.title.main {
            search_queries.push(romaji.clone());
        }
    }

    // Synonyms
    for synonym in &jikan_data.title.synonyms {
        if !synonym.is_empty() && !search_queries.contains(synonym) {
            search_queries.push(synonym.clone());
        }
    }

    // Step 3: Search AniList with each title variant
    for (i, query) in search_queries.iter().enumerate() {
        log::info!(
            "Searching AniList with variant {}/{}: '{}'",
            i + 1,
            search_queries.len(),
            query
        );

        match provider_service.search_anime(query, 10).await {
            Ok(results) => {
                // Step 4: Find best match using existing fuzzy matching logic
                let best_match = find_best_cross_provider_match(&jikan_data, &results);

                if let Some(anilist_anime) = best_match {
                    if let Some(anilist_id) = anilist_anime
                        .provider_metadata
                        .external_ids
                        .get(&AnimeProvider::AniList)
                    {
                        let anilist_id_num: u32 = anilist_id
                            .parse()
                            .map_err(|e| format!("Invalid AniList ID format: {}", e))?;

                        log::info!(
                            "✅ Found AniList match! Jikan ID {} -> AniList ID {} (matched on '{}')",
                            jikan_id, anilist_id_num, query
                        );
                        return Ok(Some(anilist_id_num));
                    }
                }
            }
            Err(e) => {
                log::warn!("AniList search failed for '{}': {}", query, e);
                continue;
            }
        }
    }

    log::info!(
        "Auto-enrichment: No AniList match found for '{}'",
        anime.title.main
    );
    Ok(None)
}

// Helper function to find Jikan ID using existing AniList data
async fn find_jikan_by_title(
    anime: &crate::modules::anime::AnimeDetailed,
    provider_service: &ProviderService,
) -> Result<Option<u32>, String> {
    use crate::modules::provider::domain::value_objects::provider_enum::AnimeProvider;

    // Get AniList ID from current provider metadata
    let anilist_id = anime
        .provider_metadata
        .external_ids
        .get(&AnimeProvider::AniList)
        .ok_or("No AniList ID found for cross-provider search")?;

    log::info!(
        "Using AniList ID {} to find Jikan match for '{}'",
        anilist_id,
        anime.title.main
    );

    // Step 1: Get full anime data from AniList to extract all title variants
    let anilist_data = match provider_service
        .get_anime_by_id(anilist_id, AnimeProvider::AniList)
        .await
    {
        Ok(Some(data)) => data,
        Ok(None) => {
            log::warn!(
                "AniList ID {} not found, cannot cross-reference",
                anilist_id
            );
            return Ok(None);
        }
        Err(e) => {
            log::error!("Failed to get AniList data for ID {}: {}", anilist_id, e);
            return Ok(None);
        }
    };

    // Step 2: Extract multiple search queries from AniList data
    let mut search_queries = Vec::new();

    // Primary titles
    search_queries.push(anilist_data.title.main.clone());
    if let Some(english) = &anilist_data.title.english {
        if !english.is_empty() && english != &anilist_data.title.main {
            search_queries.push(english.clone());
        }
    }
    if let Some(romaji) = &anilist_data.title.romaji {
        if !romaji.is_empty() && romaji != &anilist_data.title.main {
            search_queries.push(romaji.clone());
        }
    }

    // Synonyms
    for synonym in &anilist_data.title.synonyms {
        if !synonym.is_empty() && !search_queries.contains(synonym) {
            search_queries.push(synonym.clone());
        }
    }

    // Step 3: Search Jikan with each title variant
    for (i, query) in search_queries.iter().enumerate() {
        log::info!(
            "Searching Jikan with variant {}/{}: '{}'",
            i + 1,
            search_queries.len(),
            query
        );

        match provider_service.search_anime(query, 10).await {
            Ok(results) => {
                // Step 4: Find best match using existing fuzzy matching logic
                let best_match = find_best_cross_provider_match(&anilist_data, &results);

                if let Some(jikan_anime) = best_match {
                    if let Some(jikan_id) = jikan_anime
                        .provider_metadata
                        .external_ids
                        .get(&AnimeProvider::Jikan)
                    {
                        let jikan_id_num: u32 = jikan_id
                            .parse()
                            .map_err(|e| format!("Invalid Jikan ID format: {}", e))?;

                        log::info!(
                            "✅ Found Jikan match! AniList ID {} -> Jikan ID {} (matched on '{}')",
                            anilist_id,
                            jikan_id_num,
                            query
                        );
                        return Ok(Some(jikan_id_num));
                    }
                }
            }
            Err(e) => {
                log::warn!("Jikan search failed for '{}': {}", query, e);
                continue;
            }
        }
    }

    log::info!(
        "Auto-enrichment: No Jikan match found for '{}'",
        anime.title.main
    );
    Ok(None)
}

/// Find best cross-provider match using sophisticated matching logic
fn find_best_cross_provider_match<'a>(
    source_anime: &'a crate::modules::anime::AnimeDetailed,
    search_results: &'a [crate::modules::anime::AnimeDetailed],
) -> Option<&'a crate::modules::anime::AnimeDetailed> {
    if search_results.is_empty() {
        return None;
    }

    let mut best_match: Option<&crate::modules::anime::AnimeDetailed> = None;
    let mut best_score = 0.0;

    for candidate in search_results {
        let mut score = 0.0;
        let mut match_criteria = Vec::new();

        // Title similarity (most important - 40% weight)
        let title_similarity = calculate_title_similarity(&source_anime.title, &candidate.title);
        score += title_similarity * 0.4;
        if title_similarity > 0.8 {
            match_criteria.push(format!("title_sim:{:.2}", title_similarity));
        }

        // Episode count match (30% weight)
        if let (Some(source_eps), Some(candidate_eps)) = (source_anime.episodes, candidate.episodes)
        {
            if source_eps == candidate_eps {
                score += 0.3;
                match_criteria.push("episodes_match".to_string());
            } else if (source_eps as i32 - candidate_eps as i32).abs() <= 1 {
                score += 0.15; // Close episode count
                match_criteria.push("episodes_close".to_string());
            }
        }

        // Year match (20% weight)
        let source_year = source_anime.aired.from.as_ref().map(|dt| dt.year());
        let candidate_year = candidate.aired.from.as_ref().map(|dt| dt.year());

        if let (Some(src_year), Some(cand_year)) = (source_year, candidate_year) {
            if src_year == cand_year {
                score += 0.2;
                match_criteria.push("year_match".to_string());
            } else if (src_year - cand_year).abs() <= 1 {
                score += 0.1; // Close year
                match_criteria.push("year_close".to_string());
            }
        }

        // Type match (10% weight)
        if source_anime.anime_type == candidate.anime_type {
            score += 0.1;
            match_criteria.push("type_match".to_string());
        }

        log::debug!(
            "Cross-provider match candidate '{}': score={:.2}, criteria=[{}]",
            candidate.title.main,
            score,
            match_criteria.join(", ")
        );

        // Consider it a good match if score > 0.7 (70% confidence)
        if score > best_score && score > 0.7 {
            best_score = score;
            best_match = Some(candidate);
        }
    }

    if let Some(matched) = best_match {
        log::info!(
            "✅ Best cross-provider match: '{}' -> '{}' (score: {:.2})",
            source_anime.title.main,
            matched.title.main,
            best_score
        );
    }

    best_match
}

/// Calculate similarity between two anime titles
fn calculate_title_similarity(
    title1: &crate::modules::anime::domain::value_objects::AnimeTitle,
    title2: &crate::modules::anime::domain::value_objects::AnimeTitle,
) -> f64 {
    use strsim::jaro_winkler;

    let mut max_similarity: f64 = 0.0;

    // Get all title variants for both anime
    let titles1 = get_all_title_variants(title1);
    let titles2 = get_all_title_variants(title2);

    // Compare every title variant against every other
    for t1 in &titles1 {
        for t2 in &titles2 {
            let similarity = jaro_winkler(&normalize_title(t1), &normalize_title(t2));
            max_similarity = max_similarity.max(similarity);
        }
    }

    max_similarity
}

/// Get all title variants from an AnimeTitle
fn get_all_title_variants(
    title: &crate::modules::anime::domain::value_objects::AnimeTitle,
) -> Vec<String> {
    let mut variants = vec![title.main.clone()];

    if let Some(english) = &title.english {
        if !english.is_empty() {
            variants.push(english.clone());
        }
    }

    if let Some(romaji) = &title.romaji {
        if !romaji.is_empty() {
            variants.push(romaji.clone());
        }
    }

    for synonym in &title.synonyms {
        if !synonym.is_empty() {
            variants.push(synonym.clone());
        }
    }

    variants
}

/// Normalize title for better matching
fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .replace("(tv)", "")
        .replace("(movie)", "")
        .replace("(ova)", "")
        .replace("(ona)", "")
        .replace("(special)", "")
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Merge and save enriched anime data using existing data quality service
async fn merge_and_save_enriched_data(
    existing_anime: &crate::modules::anime::AnimeDetailed,
    new_provider_anime: &crate::modules::anime::AnimeDetailed,
    anime_service: &crate::modules::anime::AnimeService,
) -> Result<crate::modules::anime::AnimeDetailed, Box<dyn std::error::Error>> {
    use crate::modules::anime::domain::services::data_quality_service::DataQualityService;
    use crate::modules::provider::domain::entities::anime_data::{
        AnimeData, DataQuality, DataSource,
    };

    // Convert AnimeDetailed to AnimeData format for the data quality service
    let existing_data = AnimeData {
        anime: existing_anime.clone(),
        quality: DataQuality::calculate(existing_anime),
        source: DataSource {
            primary_provider: get_primary_provider(existing_anime),
            providers_used: get_all_providers(existing_anime),
            confidence: 0.8, // High confidence for existing database data
            fetch_time_ms: 0,
        },
    };

    let new_data = AnimeData {
        anime: new_provider_anime.clone(),
        quality: DataQuality::calculate(new_provider_anime),
        source: DataSource {
            primary_provider: get_primary_provider(new_provider_anime),
            providers_used: get_all_providers(new_provider_anime),
            confidence: 0.9, // High confidence for provider data
            fetch_time_ms: 0,
        },
    };

    // Use the existing data quality service to merge intelligently
    let data_quality_service = DataQualityService::new();
    let merged_data = data_quality_service.merge_anime_data(vec![existing_data, new_data])?;

    // Ensure the merged anime keeps the same ID as the existing one
    let mut merged_anime = merged_data.anime;
    merged_anime.id = existing_anime.id.clone();

    // Save the merged anime using the service's new save_anime method
    let saved_anime = anime_service.save_anime(&merged_anime).await?;

    log::info!(
        "✅ Successfully merged and saved anime '{}' with {} providers",
        saved_anime.title.main,
        saved_anime.provider_metadata.external_ids.len()
    );

    Ok(saved_anime)
}

/// Get the primary provider for an anime based on its external IDs
fn get_primary_provider(
    anime: &crate::modules::anime::AnimeDetailed,
) -> crate::modules::provider::domain::value_objects::provider_enum::AnimeProvider {
    use crate::modules::provider::domain::value_objects::provider_enum::AnimeProvider;

    let external_ids = &anime.provider_metadata.external_ids;

    // Priority: AniList > Jikan (based on data quality and features)
    if external_ids.contains_key(&AnimeProvider::AniList) {
        AnimeProvider::AniList
    } else if external_ids.contains_key(&AnimeProvider::Jikan) {
        AnimeProvider::Jikan
    } else {
        // Default fallback
        AnimeProvider::Jikan
    }
}

/// Get all providers that have data for this anime
fn get_all_providers(
    anime: &crate::modules::anime::AnimeDetailed,
) -> Vec<crate::modules::provider::domain::value_objects::provider_enum::AnimeProvider> {
    anime
        .provider_metadata
        .external_ids
        .keys()
        .cloned()
        .collect()
}
