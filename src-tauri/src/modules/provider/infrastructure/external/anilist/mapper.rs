use crate::modules::anime::domain::{
    entities::{
        anime_detailed::{AiredDates, AnimeDetailed},
        genre::Genre,
    },
    value_objects::{AnimeStatus, AnimeTier, AnimeTitle, AnimeType, QualityMetrics},
};
use crate::modules::anime::UnifiedAgeRestriction;
use crate::modules::provider::domain::{AnimeProvider, ProviderMetadata};

use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use super::dto::{AniListDate, AniListMedia, AniListTag};

pub struct AniListMapper;

impl AniListMapper {
    pub fn to_domain(anilist_media: AniListMedia) -> AnimeDetailed {
        let id = Uuid::new_v4();

        // Map title
        let title = Self::map_title(&anilist_media.title);

        // Map dates
        let (aired_from, aired_to) =
            Self::map_dates(&anilist_media.start_date, &anilist_media.end_date);
        let aired = AiredDates {
            from: aired_from,
            to: aired_to,
        };

        // Map genres
        let genres = anilist_media
            .genres
            .iter()
            .map(|genre_name| Genre {
                id: Uuid::new_v4(),
                name: genre_name.clone(),
            })
            .collect();

        // Map studios
        let studios = anilist_media
            .studios
            .as_ref()
            .map(|studio_conn| {
                studio_conn
                    .edges
                    .iter()
                    .map(|edge| edge.node.name.clone())
                    .collect()
            })
            .unwrap_or_default();

        // Map external provider metadata
        let mut external_ids = HashMap::new();
        external_ids.insert(AnimeProvider::AniList, anilist_media.id.to_string());

        if let Some(mal_id) = anilist_media.id_mal {
            external_ids.insert(AnimeProvider::Jikan, mal_id.to_string());
        }

        let provider_metadata = ProviderMetadata {
            external_ids,
            provider_urls: HashMap::new(),
            user_preferred_provider: Some(AnimeProvider::AniList),
            primary_provider: AnimeProvider::AniList,
        };

        // Calculate quality metrics from AniList data
        let quality_metrics = Self::calculate_quality_metrics(&anilist_media);

        // Map image URL
        let image_url = anilist_media
            .cover_image
            .clone()
            .and_then(|img| img.extra_large.or(img.large.or(img.medium)));

        AnimeDetailed {
            id,
            title,
            provider_metadata,
            score: Self::normalize_score(anilist_media.average_score), // Unified 0-10 scale
            favorites: anilist_media.favourites.map(|f| f as u32),     // Primary engagement metric
            synopsis: anilist_media.description.clone(),
            episodes: anilist_media.episodes.map(|e| e as u16),
            status: Self::map_status(anilist_media.status.as_deref()),
            aired,
            anime_type: Self::map_anime_type(anilist_media.format.as_deref()),
            age_restriction: Some(Self::map_age_restriction(
                anilist_media.is_adult,
                &anilist_media.tags,
            )),
            genres,
            studios,
            source: anilist_media.source,
            duration: anilist_media.duration.map(|d| format!("{} min", d)),
            image_url,
            banner_image: anilist_media.banner_image,
            trailer_url: anilist_media.trailer.and_then(|t| match t.site.as_deref() {
                Some("youtube") => {
                    t.id.map(|id| format!("https://www.youtube.com/watch?v={}", id))
                }
                Some("dailymotion") => {
                    t.id.map(|id| format!("https://www.dailymotion.com/video/{}", id))
                }
                _ => None,
            }),
            composite_score: 0.0, // Will be calculated later
            tier: AnimeTier::C,   // Default tier, will be calculated later
            quality_metrics,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_synced_at: Some(Utc::now()),
        }
    }

    /// Normalize AniList score (0-100) to unified 0-10 scale
    fn normalize_score(score: Option<i32>) -> Option<f32> {
        score.map(|s| (s as f32 / 10.0).clamp(0.0, 10.0))
    }

    fn map_title(
        anilist_title: &crate::modules::provider::infrastructure::external::anilist::dto::AniListTitle,
    ) -> AnimeTitle {
        AnimeTitle {
            main: anilist_title
                .user_preferred
                .as_ref()
                .or(anilist_title.romaji.as_ref())
                .or(anilist_title.english.as_ref())
                .cloned()
                .unwrap_or_else(|| "Unknown Title".to_string()),
            english: anilist_title.english.clone(),
            japanese: anilist_title.native.clone(),
            romaji: anilist_title.romaji.clone(),
            native: anilist_title.native.clone(),
            synonyms: vec![], // AniList synonyms would need to be mapped separately
        }
    }

    fn map_dates(
        start_date: &Option<AniListDate>,
        end_date: &Option<AniListDate>,
    ) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
        let aired_from = start_date.as_ref().and_then(Self::anilist_date_to_chrono);
        let aired_to = end_date.as_ref().and_then(Self::anilist_date_to_chrono);
        (aired_from, aired_to)
    }

    fn anilist_date_to_chrono(anilist_date: &AniListDate) -> Option<DateTime<Utc>> {
        let year = anilist_date.year?;
        let month = anilist_date.month.unwrap_or(1);
        let day = anilist_date.day.unwrap_or(1);

        Utc.with_ymd_and_hms(year, month as u32, day as u32, 0, 0, 0)
            .single()
    }

    fn map_status(status: Option<&str>) -> AnimeStatus {
        match status {
            Some("FINISHED") => AnimeStatus::Finished,
            Some("RELEASING") => AnimeStatus::Airing,
            Some("NOT_YET_RELEASED") => AnimeStatus::NotYetAired,
            Some("CANCELLED") => AnimeStatus::Cancelled,
            Some("HIATUS") => AnimeStatus::Unknown, // Map HIATUS to Unknown since we don't have OnHiatus
            _ => AnimeStatus::Unknown,
        }
    }

    fn map_anime_type(format: Option<&str>) -> AnimeType {
        match format {
            Some("TV") => AnimeType::TV,
            Some("MOVIE") => AnimeType::Movie,
            Some("OVA") => AnimeType::OVA,
            Some("ONA") => AnimeType::ONA,
            Some("SPECIAL") => AnimeType::Special,
            Some("TV_SHORT") => AnimeType::TV, // Map TV_SHORT to TV for simplicity
            _ => AnimeType::Unknown,
        }
    }

    fn map_age_restriction(
        is_adult: Option<bool>,
        tags: &Option<Vec<AniListTag>>,
    ) -> UnifiedAgeRestriction {
        // If explicitly marked as adult content
        if matches!(is_adult, Some(true)) {
            return UnifiedAgeRestriction::Explicit;
        }

        // Check tags for mature content indicators
        if let Some(tag_list) = tags {
            let has_mature_themes = tag_list.iter().any(|tag| {
                let tag_name = tag.name.to_lowercase();
                // Check for explicit adult tags
                if matches!(tag.is_adult, Some(true)) {
                    return true;
                }
                // Check for mature content indicators
                matches!(
                    tag_name.as_str(),
                    "violence"
                        | "gore"
                        | "nudity"
                        | "sexual themes"
                        | "mature themes"
                        | "ecchi"
                        | "sexual content"
                        | "graphic violence"
                        | "blood"
                        | "partial nudity"
                        | "suggestive themes"
                        | "mild sexual themes"
                )
            });

            let has_teen_themes = tag_list.iter().any(|tag| {
                let tag_name = tag.name.to_lowercase();
                matches!(
                    tag_name.as_str(),
                    "mild violence"
                        | "comedy"
                        | "romance"
                        | "school"
                        | "slice of life"
                        | "sports"
                        | "supernatural"
                        | "fantasy"
                        | "sci-fi"
                        | "adventure"
                )
            });

            // Return appropriate rating based on tag analysis
            return match (has_mature_themes, has_teen_themes) {
                (true, _) => UnifiedAgeRestriction::Mature,
                (false, true) => UnifiedAgeRestriction::ParentalGuidance13,
                (false, false) => UnifiedAgeRestriction::ParentalGuidance13, // Default to PG-13 for consistency
            };
        }

        // Default to PG-13 for consistency with Jikan when no tags available
        UnifiedAgeRestriction::ParentalGuidance13
    }

    fn calculate_quality_metrics(anilist_media: &AniListMedia) -> QualityMetrics {
        // Calculate popularity score (0.0 to 1.0)
        let popularity_score = anilist_media
            .popularity
            .map(|p| (p as f32 / 500000.0).min(1.0)) // Normalize based on high popularity anime
            .unwrap_or(0.0);

        // Calculate engagement score based on score and favourites
        let score_component = anilist_media
            .average_score
            .map(|s| s as f32 / 100.0) // Convert from 0-100 to 0-1
            .unwrap_or(0.0);

        let favourites_component = anilist_media
            .favourites
            .map(|f| (f as f32 / 100000.0).min(1.0)) // Normalize favourites
            .unwrap_or(0.0);

        let engagement_score = (score_component * 0.6 + favourites_component * 0.4).min(1.0);

        // Calculate consistency score (based on whether anime is complete and well-rated)
        let consistency_score = match (anilist_media.status.as_deref(), anilist_media.average_score)
        {
            (Some("FINISHED"), Some(score)) if score >= 70 => 0.9,
            (Some("FINISHED"), Some(score)) if score >= 60 => 0.7,
            (Some("FINISHED"), _) => 0.6,
            (Some("RELEASING"), Some(score)) if score >= 75 => 0.8,
            (Some("RELEASING"), _) => 0.5,
            _ => 0.3,
        };

        // Audience reach score combines popularity and score
        let audience_reach_score = (popularity_score * 0.7 + engagement_score * 0.3).min(1.0);

        QualityMetrics {
            popularity_score,
            engagement_score,
            consistency_score,
            audience_reach_score,
        }
    }
}
