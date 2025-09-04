use crate::domain::{
    entities::{AiredDates, Anime, Genre},
    services::ScoreCalculator,
    value_objects::{AnimeStatus, AnimeTier, AnimeType, QualityMetrics},
};
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use super::dto::{JikanAnimeData, JikanEntity};

pub struct JikanMapper;

impl JikanMapper {
    pub fn to_domain(dto: JikanAnimeData) -> Anime {
        let mut anime = Anime {
            id: Uuid::new_v4(),
            mal_id: Some(dto.mal_id),
            title: dto.title.clone(),
            title_english: dto.title_english.clone(),
            title_japanese: dto.title_japanese.clone(),
            score: dto.score,
            scored_by: dto.scored_by,
            rank: dto.rank,
            popularity: dto.popularity,
            members: dto.members,
            favorites: dto.favorites,
            synopsis: dto.synopsis.clone(),
            episodes: dto.episodes,
            status: Self::map_status(dto.status.as_deref()),
            aired: Self::map_aired_dates(&dto.aired),
            anime_type: Self::map_type(dto.anime_type.as_deref()),
            rating: dto.rating.clone(),
            genres: Self::map_genres(&dto.genres),
            studios: Self::map_studios(&dto.studios),
            source: dto.source.clone(),
            duration: dto.duration.clone(),
            image_url: Self::extract_image_url(&dto),
            mal_url: Some(dto.url.clone()),
            composite_score: 0.0,
            tier: AnimeTier::default(),
            quality_metrics: QualityMetrics::default(),
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
            .map(|g| Genre {
                id: Uuid::new_v4(),
                mal_id: Some(g.mal_id),
                name: g.name.clone(),
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
