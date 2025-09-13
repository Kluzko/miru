use crate::domain::{
    entities::{AiredDates, AnimeDetailed, Genre},
    services::ScoreCalculator,
    value_objects::{
        AnimeProvider, AnimeStatus, AnimeTier, AnimeTitle, AnimeType, ProviderMetadata,
        QualityMetrics, UnifiedAgeRestriction,
    },
};
use crate::infrastructure::shared::mappers::age_restriction_mapper::AgeRestrictionMapper;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use super::dto::{JikanAnimeData, JikanEntity};

pub struct JikanMapper;

impl JikanMapper {
    pub fn to_domain(dto: JikanAnimeData) -> AnimeDetailed {
        // Generate a deterministic UUID based on mal_id if available
        // This ensures the same anime always gets the same UUID in our system
        let id = if dto.mal_id > 0 {
            // Use namespace UUID to generate deterministic IDs
            let namespace = Uuid::NAMESPACE_OID;
            Uuid::new_v5(&namespace, format!("mal_anime_{}", dto.mal_id).as_bytes())
        } else {
            // Fallback to random UUID for anime without mal_id
            Uuid::new_v4()
        };

        // Create title with all variants
        let title = AnimeTitle::with_variants(
            dto.title.clone(),
            dto.title_english.clone(),
            dto.title_japanese.clone(),
            None, // romaji not provided by Jikan
        );

        // Create provider metadata with Jikan as primary provider
        let mut provider_metadata =
            ProviderMetadata::new(AnimeProvider::Jikan, dto.mal_id.to_string());
        provider_metadata.add_provider_url(AnimeProvider::Jikan, dto.url.clone());

        let mut anime = AnimeDetailed {
            id,
            title,
            provider_metadata,
            score: dto.score,
            scored_by: dto.scored_by.map(|v| v as u32),
            rank: dto.rank.map(|v| v as u32),
            popularity: dto.popularity.map(|v| v as u32),
            members: dto.members.map(|v| v as u32),
            favorites: dto.favorites.map(|v| v as u32),
            synopsis: dto.synopsis.clone(),
            episodes: dto.episodes.map(|v| v as u16),
            status: Self::map_status(dto.status.as_deref()),
            aired: Self::map_aired_dates(&dto.aired),
            anime_type: Self::map_type(dto.anime_type.as_deref()),
            age_restriction: Self::map_rating(&dto.rating),
            genres: Self::map_genres(&dto.genres),
            studios: Self::map_studios(&dto.studios),
            source: dto.source.clone(),
            duration: dto.duration.clone(),
            image_url: Self::extract_image_url(&dto),
            banner_image: None,
            trailer_url: None,
            composite_score: 0.0,
            tier: AnimeTier::default(),
            quality_metrics: QualityMetrics::default(),
            // episodes_list: Vec::new(), // Removed - field deleted
            // relations: Vec::new(),     // Removed - field deleted
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Calculate scores
        let calculator = ScoreCalculator::new();
        anime.update_scores(&calculator);

        anime
    }

    fn map_status(status: Option<&str>) -> AnimeStatus {
        match status {
            Some("Currently Airing") => AnimeStatus::Airing,
            Some("Finished Airing") => AnimeStatus::Finished,
            Some("Not yet aired") => AnimeStatus::NotYetAired,
            _ => AnimeStatus::Unknown,
        }
    }

    fn map_type(anime_type: Option<&str>) -> AnimeType {
        match anime_type {
            Some("TV") => AnimeType::TV,
            Some("Movie") => AnimeType::Movie,
            Some("OVA") => AnimeType::OVA,
            Some("Special") => AnimeType::Special,
            Some("ONA") => AnimeType::ONA,
            Some("Music") => AnimeType::Music,
            _ => AnimeType::Unknown,
        }
    }

    fn map_rating(rating: &Option<String>) -> Option<UnifiedAgeRestriction> {
        rating
            .as_ref()
            .and_then(|r| AgeRestrictionMapper::map_to_unified(&AnimeProvider::Jikan, r))
    }

    fn map_aired_dates(aired: &super::dto::JikanAired) -> AiredDates {
        AiredDates {
            from: aired.from.as_ref().and_then(|s| Self::parse_date(s)),
            to: aired.to.as_ref().and_then(|s| Self::parse_date(s)),
        }
    }

    fn parse_date(date_str: &str) -> Option<DateTime<Utc>> {
        // Try parsing ISO 8601 format first
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Some(dt.with_timezone(&Utc));
        }

        // Try parsing as datetime with timezone
        if let Ok(dt) = date_str.parse::<DateTime<Utc>>() {
            return Some(dt);
        }

        // Try parsing as date only (YYYY-MM-DD)
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return Some(
                date.and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap(),
            );
        }

        // Try other common formats
        let formats = [
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%dT%H:%M:%S",
            "%Y/%m/%d",
            "%d-%m-%Y",
            "%d/%m/%Y",
        ];

        for format in &formats {
            if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
                return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
            }
            if let Ok(naive_date) = NaiveDate::parse_from_str(date_str, format) {
                return Some(
                    naive_date
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_local_timezone(Utc)
                        .unwrap(),
                );
            }
        }

        None
    }

    fn map_genres(genres: &[JikanEntity]) -> Vec<Genre> {
        genres
            .iter()
            .map(|g| {
                // Generate deterministic UUID for genres based on mal_id
                let id = if g.mal_id > 0 {
                    let namespace = Uuid::NAMESPACE_OID;
                    Uuid::new_v5(&namespace, format!("mal_genre_{}", g.mal_id).as_bytes())
                } else {
                    // Use name-based UUID for consistency
                    let namespace = Uuid::NAMESPACE_OID;
                    Uuid::new_v5(&namespace, format!("genre_{}", g.name).as_bytes())
                };

                Genre {
                    id,
                    name: g.name.clone(),
                }
            })
            .collect()
    }

    fn map_studios(studios: &[JikanEntity]) -> Vec<String> {
        studios.iter().map(|s| s.name.clone()).collect()
    }

    fn extract_image_url(dto: &JikanAnimeData) -> Option<String> {
        dto.images
            .jpg
            .large_image_url
            .clone()
            .or_else(|| dto.images.jpg.image_url.clone())
    }
}
