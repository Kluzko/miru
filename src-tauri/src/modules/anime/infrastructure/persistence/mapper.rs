/// Shared mapping utilities for converting between database models and domain entities
///
/// This module contains all conversion logic used by repository implementations.
/// Separated for reusability and to avoid code duplication.
use chrono::Utc;
use std::collections::HashMap;

use crate::modules::anime::domain::{
    entities::{
        anime_detailed::{AiredDates, AnimeDetailed},
        genre::Genre,
    },
    value_objects::{anime_title::AnimeTitle, quality_metrics::QualityMetrics},
};
use crate::modules::anime::infrastructure::models::*;
use crate::shared::domain::value_objects::{AnimeProvider, ProviderMetadata};

/// Convert database Anime model to AnimeDetailed entity (simple version without external IDs)
pub fn model_to_entity(
    model: Anime,
    genres: Vec<Genre>,
    studios: Vec<String>,
    quality_metrics: Option<QualityMetrics>,
) -> AnimeDetailed {
    // Create AnimeTitle from database fields
    let mut title = AnimeTitle::with_variants(
        model.title_main,
        model.title_english,
        model.title_japanese,
        model.title_romaji,
    );

    // Set native title and synonyms
    title.native = model.title_native;
    title.synonyms = model
        .title_synonyms
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .unwrap_or_default();

    // Create ProviderMetadata - we'll populate from external_ids table later
    // For now, create minimal metadata with Jikan as default
    let provider_metadata = ProviderMetadata::new(
        AnimeProvider::Jikan, // Default provider
        "0".to_string(),      // Will be populated from external_ids table
    );

    AnimeDetailed {
        id: model.id,
        title,
        provider_metadata,
        score: model.score,
        rating: model.score, // Alias for score
        favorites: model.favorites.map(|v| v as u32),
        synopsis: model.synopsis.clone(),
        description: model.synopsis, // Alias for synopsis
        episodes: model.episodes.map(|v| v as u16),
        status: model.status,
        aired: AiredDates {
            from: model.aired_from,
            to: model.aired_to,
        },
        anime_type: model.anime_type,
        age_restriction: model.age_restriction,
        genres,
        studios,
        source: model.source,
        duration: model.duration,
        image_url: model.image_url.clone(),
        images: model.image_url, // Alias for image_url
        banner_image: model.banner_image,
        trailer_url: model.trailer_url,
        composite_score: model.composite_score,
        tier: model.tier,
        quality_metrics: quality_metrics.unwrap_or_default(),
        created_at: model.created_at,
        updated_at: model.updated_at,
        last_synced_at: model.last_synced_at,
    }
}

/// Convert database model to entity with proper external IDs
pub fn model_to_entity_with_external_ids(
    model: Anime,
    genres: Vec<Genre>,
    studios: Vec<String>,
    quality_metrics: Option<QualityMetrics>,
    external_ids: HashMap<AnimeProvider, String>,
) -> AnimeDetailed {
    // Create AnimeTitle from database fields
    let mut title = AnimeTitle::with_variants(
        model.title_main,
        model.title_english,
        model.title_japanese,
        model.title_romaji,
    );

    // Set native title and synonyms
    title.native = model.title_native;
    title.synonyms = model
        .title_synonyms
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .unwrap_or_default();

    // Create ProviderMetadata with actual external IDs
    let provider_metadata = if external_ids.is_empty() {
        // Fallback to default if no external IDs found
        ProviderMetadata::new(AnimeProvider::Jikan, "0".to_string())
    } else {
        // Use the first available provider as primary, preferring AniList
        let (primary_provider, primary_id) =
            if let Some(anilist_id) = external_ids.get(&AnimeProvider::AniList) {
                (AnimeProvider::AniList, anilist_id.clone())
            } else if let Some(jikan_id) = external_ids.get(&AnimeProvider::Jikan) {
                (AnimeProvider::Jikan, jikan_id.clone())
            } else {
                // Use the first available provider
                let (provider, id) = external_ids.iter().next().unwrap();
                (provider.clone(), id.clone())
            };

        let mut metadata = ProviderMetadata::new(primary_provider, primary_id);
        metadata.external_ids = external_ids;
        metadata
    };

    AnimeDetailed {
        id: model.id,
        title,
        provider_metadata,
        score: model.score,
        rating: model.score, // Alias for score
        favorites: model.favorites.map(|v| v as u32),
        synopsis: model.synopsis.clone(),
        description: model.synopsis, // Alias for synopsis
        episodes: model.episodes.map(|v| v as u16),
        status: model.status,
        aired: AiredDates {
            from: model.aired_from,
            to: model.aired_to,
        },
        anime_type: model.anime_type,
        age_restriction: model.age_restriction,
        genres,
        studios,
        source: model.source,
        duration: model.duration,
        image_url: model.image_url.clone(),
        images: model.image_url, // Alias for image_url
        banner_image: model.banner_image,
        trailer_url: model.trailer_url,
        composite_score: model.composite_score,
        tier: model.tier,
        quality_metrics: quality_metrics.unwrap_or_default(),
        created_at: model.created_at,
        updated_at: model.updated_at,
        last_synced_at: model.last_synced_at,
    }
}

/// Convert AnimeDetailed to NewAnime for insertion
pub fn entity_to_new_model(entity: &AnimeDetailed) -> NewAnime {
    log::info!(
        "DB SAVE: Converting AnimeDetailed '{}' to NewAnime - age_restriction: {:?}",
        entity.title.main,
        entity.age_restriction
    );

    NewAnime {
        id: entity.id,
        title_english: entity.title.english.clone(),
        title_japanese: entity.title.japanese.clone(),
        score: entity.score,
        favorites: entity.favorites.map(|v| v as i32),
        synopsis: entity.synopsis.clone(),
        episodes: entity.episodes.map(|v| v as i32),
        aired_from: entity.aired.from,
        aired_to: entity.aired.to,
        source: entity.source.clone(),
        duration: entity.duration.clone(),
        image_url: entity.image_url.clone(),
        composite_score: entity.composite_score,
        title_main: entity.title.main.clone(),
        title_romaji: entity.title.romaji.clone(),
        title_native: entity.title.native.clone(),
        title_synonyms: if entity.title.synonyms.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&entity.title.synonyms).unwrap_or(serde_json::Value::Null))
        },
        banner_image: entity.banner_image.clone(),
        trailer_url: entity.trailer_url.clone(),
        tier: entity.tier.clone(),
        quality_metrics: Some(
            serde_json::to_value(&entity.quality_metrics).unwrap_or(serde_json::Value::Null),
        ),
        age_restriction: {
            let age_restriction = entity.age_restriction.clone();
            log::info!(
                "DB SAVE: NewAnime age_restriction field set to: {:?}",
                age_restriction
            );
            age_restriction
        },
        status: entity.status.clone(),
        anime_type: entity.anime_type.clone(),
        last_synced_at: entity.last_synced_at,
    }
}

/// Convert AnimeDetailed to AnimeChangeset for updates
pub fn entity_to_changeset(entity: &AnimeDetailed) -> AnimeChangeset {
    AnimeChangeset {
        title_english: entity.title.english.clone(),
        title_japanese: entity.title.japanese.clone(),
        score: entity.score,
        favorites: entity.favorites.map(|v| v as i32),
        synopsis: entity.synopsis.clone(),
        episodes: entity.episodes.map(|v| v as i32),
        aired_from: entity.aired.from,
        aired_to: entity.aired.to,
        source: entity.source.clone(),
        duration: entity.duration.clone(),
        image_url: entity.image_url.clone(),
        composite_score: entity.composite_score,
        updated_at: Utc::now(),
        title_main: entity.title.main.clone(),
        title_romaji: entity.title.romaji.clone(),
        title_native: entity.title.native.clone(),
        title_synonyms: if entity.title.synonyms.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&entity.title.synonyms).unwrap_or(serde_json::Value::Null))
        },
        banner_image: entity.banner_image.clone(),
        trailer_url: entity.trailer_url.clone(),
        tier: entity.tier.clone(),
        quality_metrics: Some(
            serde_json::to_value(&entity.quality_metrics).unwrap_or(serde_json::Value::Null),
        ),
        age_restriction: entity.age_restriction.clone(),
        status: entity.status.clone(),
        anime_type: entity.anime_type.clone(),
        last_synced_at: entity.last_synced_at,
    }
}
