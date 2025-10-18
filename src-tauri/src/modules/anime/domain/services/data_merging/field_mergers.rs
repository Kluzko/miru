use super::merge_context::MergeContext;
use crate::modules::anime::AnimeDetailed;

/// Field-specific mergers following Single Responsibility Principle
/// Each merger handles one category of fields

/// Trait for field-specific merging logic
pub trait FieldMerger {
    /// Merge fields into the target anime
    fn merge_into(&self, target: &mut AnimeDetailed, context: &MergeContext);
}

/// Merges title-related fields (english, japanese, romaji, native, synonyms)
#[derive(Debug, Clone, Copy)]
pub struct TitleMerger;

impl FieldMerger for TitleMerger {
    fn merge_into(&self, target: &mut AnimeDetailed, context: &MergeContext) {
        for source in &context.sources {
            // Fill missing title variants
            if target.title.english.is_none() {
                target.title.english = source.anime.title.english.clone();
            }
            if target.title.japanese.is_none() {
                target.title.japanese = source.anime.title.japanese.clone();
            }
            if target.title.romaji.is_none() {
                target.title.romaji = source.anime.title.romaji.clone();
            }
            if target.title.native.is_none() {
                target.title.native = source.anime.title.native.clone();
            }

            // Merge synonyms (deduplicate)
            for synonym in &source.anime.title.synonyms {
                if !target.title.synonyms.contains(synonym) {
                    target.title.synonyms.push(synonym.clone());
                }
            }
        }
    }
}

/// Merges metadata fields (description, source, duration, status, type)
#[derive(Debug, Clone, Copy)]
pub struct MetadataMerger;

impl FieldMerger for MetadataMerger {
    fn merge_into(&self, target: &mut AnimeDetailed, context: &MergeContext) {
        for source in &context.sources {
            // Description: prefer longer, more detailed
            if let Some(source_desc) = &source.anime.description {
                match &target.description {
                    None => target.description = Some(source_desc.clone()),
                    Some(target_desc) if source_desc.len() > target_desc.len() => {
                        target.description = Some(source_desc.clone());
                    }
                    _ => {}
                }
            }

            // Synopsis (same logic)
            if let Some(source_syn) = &source.anime.synopsis {
                match &target.synopsis {
                    None => target.synopsis = Some(source_syn.clone()),
                    Some(target_syn) if source_syn.len() > target_syn.len() => {
                        target.synopsis = Some(source_syn.clone());
                    }
                    _ => {}
                }
            }

            // Simple field filling
            if target.source.is_none() {
                target.source = source.anime.source.clone();
            }
            if target.duration.is_none() {
                target.duration = source.anime.duration.clone();
            }
            if target.episodes.is_none() {
                target.episodes = source.anime.episodes;
            }
            if target.aired.from.is_none() {
                target.aired.from = source.anime.aired.from;
            }
            if target.aired.to.is_none() {
                target.aired.to = source.anime.aired.to;
            }

            // Status and type (if empty/unknown)
            use crate::modules::anime::domain::value_objects::{AnimeStatus, AnimeType};
            if target.status == AnimeStatus::Unknown && source.anime.status != AnimeStatus::Unknown
            {
                target.status = source.anime.status.clone();
            }
            if target.anime_type == AnimeType::Unknown
                && source.anime.anime_type != AnimeType::Unknown
            {
                target.anime_type = source.anime.anime_type.clone();
            }
        }
    }
}

/// Merges collection fields (genres, studios)
#[derive(Debug, Clone, Copy)]
pub struct CollectionMerger;

impl CollectionMerger {
    /// Normalize studio name for comparison
    /// Handles case-insensitive matching and common variations
    fn normalize_studio_name(name: &str) -> String {
        name.to_lowercase()
            .trim()
            .replace(".", "") // "J.C.STAFF" -> "jcstaff"
            .replace("-", "") // "A-1 Pictures" -> "a1 pictures"
            .replace("_", "")
            .replace(" ", "") // Remove all spaces for strict comparison
    }

    /// Remove duplicate studios that differ only in case or punctuation
    /// Called after all merging is complete to clean up any pre-existing duplicates
    pub fn deduplicate_studios(studios: &mut Vec<String>) {
        let mut seen_normalized = std::collections::HashSet::new();
        let mut deduped = Vec::new();

        for studio in studios.iter() {
            let normalized = Self::normalize_studio_name(studio);
            if seen_normalized.insert(normalized) {
                deduped.push(studio.clone());
            } else {
                log::debug!("CLEANUP: Removed duplicate studio variant: '{}'", studio);
            }
        }

        *studios = deduped;
    }
}

impl FieldMerger for CollectionMerger {
    fn merge_into(&self, target: &mut AnimeDetailed, context: &MergeContext) {
        for source in &context.sources {
            // Merge genres (deduplicate by name)
            for genre in &source.anime.genres {
                if !target.genres.iter().any(|g| g.name == genre.name) {
                    target.genres.push(genre.clone());
                }
            }

            // Merge studios (deduplicate with case-insensitive and normalization)
            if target.studios.is_empty() {
                target.studios = source.anime.studios.clone();
                log::info!(
                    "MERGE: Added studios from {:?}: {:?}",
                    source.source.primary_provider,
                    target.studios
                );
            } else {
                for studio in &source.anime.studios {
                    // Case-insensitive duplicate check with normalization
                    let studio_normalized = Self::normalize_studio_name(studio);
                    let is_duplicate = target
                        .studios
                        .iter()
                        .any(|existing| Self::normalize_studio_name(existing) == studio_normalized);

                    if !is_duplicate {
                        target.studios.push(studio.clone());
                        log::debug!(
                            "MERGE: Added studio '{}' from {:?}",
                            studio,
                            source.source.primary_provider
                        );
                    } else {
                        log::trace!(
                            "MERGE: Skipped duplicate studio '{}' (already exists as case variant)",
                            studio
                        );
                    }
                }
            }
        }

        // Final cleanup: deduplicate any existing case variants in target
        Self::deduplicate_studios(&mut target.studios);
    }
}

/// Merges rating fields (score, age_restriction, favorites)
#[derive(Debug, Clone, Copy)]
pub struct RatingMerger;

impl FieldMerger for RatingMerger {
    fn merge_into(&self, target: &mut AnimeDetailed, context: &MergeContext) {
        // Age restriction: prefer from specified provider (typically Jikan)
        if target.age_restriction.is_none() {
            if let Some(preferred_data) = context
                .get_from_preferred_provider(context.provider_preferences.age_rating_provider)
            {
                if preferred_data.anime.age_restriction.is_some() {
                    target.age_restriction = preferred_data.anime.age_restriction.clone();
                    log::info!(
                        "MERGE: Using age_restriction from preferred provider {:?}: {:?}",
                        preferred_data.source.primary_provider,
                        target.age_restriction
                    );
                }
            }

            // Fallback: use any available age restriction
            if target.age_restriction.is_none() {
                for source in &context.sources {
                    if source.anime.age_restriction.is_some() {
                        target.age_restriction = source.anime.age_restriction.clone();
                        log::info!(
                            "MERGE: Using age_restriction from {:?}: {:?}",
                            source.source.primary_provider,
                            target.age_restriction
                        );
                        break;
                    }
                }
            }
        }

        // Score: weighted average based on favorites
        let mut total_weighted_score = 0.0f32;
        let mut total_weight = 0.0f32;

        if let Some(target_score) = target.score {
            let weight = target.favorites.unwrap_or(100) as f32;
            total_weighted_score += target_score * weight;
            total_weight += weight;
        }

        for source in &context.sources {
            if let Some(source_score) = source.anime.score {
                let weight = source.anime.favorites.unwrap_or(100) as f32;
                total_weighted_score += source_score * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            let merged_score = total_weighted_score / total_weight;
            target.score = Some((merged_score * 100.0).round() / 100.0);
            target.rating = target.score; // Keep in sync
        }

        // Favorites: sum from all sources
        let mut total_favorites = target.favorites.unwrap_or(0);
        for source in &context.sources {
            total_favorites += source.anime.favorites.unwrap_or(0);
        }
        if total_favorites > 0 {
            target.favorites = Some(total_favorites);
        }
    }
}

/// Merges media fields (images, trailers, banners)
#[derive(Debug, Clone, Copy)]
pub struct MediaMerger;

impl FieldMerger for MediaMerger {
    fn merge_into(&self, target: &mut AnimeDetailed, context: &MergeContext) {
        use crate::shared::domain::value_objects::AnimeProvider;

        // Images: prefer from specified provider (typically AniList for quality)
        if target.image_url.is_none() {
            if let Some(preferred_data) =
                context.get_from_preferred_provider(context.provider_preferences.image_provider)
            {
                if preferred_data.anime.image_url.is_some() {
                    target.image_url = preferred_data.anime.image_url.clone();
                    target.images = target.image_url.clone();
                    log::debug!(
                        "MERGE: Using image from preferred provider {:?}",
                        preferred_data.source.primary_provider
                    );
                }
            }
        }

        // Fallback: use any available image
        if target.image_url.is_none() {
            for source in &context.sources {
                if source.anime.image_url.is_some() {
                    target.image_url = source.anime.image_url.clone();
                    target.images = target.image_url.clone();
                    break;
                }
            }
        }

        // Banner: only AniList provides this
        if target.banner_image.is_none() {
            for source in &context.sources {
                if source.source.primary_provider == AnimeProvider::AniList {
                    if source.anime.banner_image.is_some() {
                        target.banner_image = source.anime.banner_image.clone();
                        break;
                    }
                }
            }
        }

        // Trailer: prefer any available
        if target.trailer_url.is_none() {
            for source in &context.sources {
                if source.anime.trailer_url.is_some() {
                    target.trailer_url = source.anime.trailer_url.clone();
                    break;
                }
            }
        }
    }
}
