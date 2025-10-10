use super::models::*;
use crate::modules::anime::domain::{
    entities::{
        anime_detailed::{AiredDates, AnimeDetailed},
        genre::Genre,
    },
    value_objects::{AnimeStatus, AnimeTier, AnimeTitle, AnimeType, QualityMetrics},
};
use crate::modules::provider::domain::{
    entities::anime_data::{AnimeData, DataQuality, DataSource},
    value_objects::provider_metadata::ProviderMetadata,
    AnimeProvider,
};

use crate::shared::domain::value_objects::UnifiedAgeRestriction;
use crate::shared::errors::AppError;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Main mapper trait for converting provider-specific data to domain AnimeData
pub trait AnimeMapper<T> {
    /// Map provider data to domain AnimeData
    fn map_to_anime_data(&self, source: T) -> Result<AnimeData, AppError>;

    /// Map a list of provider data to domain AnimeData
    fn map_to_anime_data_list(&self, sources: Vec<T>) -> Result<Vec<AnimeData>, AppError> {
        sources
            .into_iter()
            .map(|source| self.map_to_anime_data(source))
            .collect()
    }
}

/// Capability trait to describe what each adapter can provide
pub trait AdapterCapabilities {
    /// Get the name of the adapter
    fn name(&self) -> &'static str;

    /// Get the provider fields this adapter can populate
    fn supported_fields(&self) -> Vec<&'static str>;

    /// Get the provider fields this adapter cannot populate
    fn unsupported_fields(&self) -> Vec<&'static str>;

    /// Check if the adapter supports a specific field
    fn supports_field(&self, field: &str) -> bool {
        self.supported_fields().contains(&field)
    }

    /// Get quality score for this adapter (0.0 to 1.0)
    fn quality_score(&self) -> f64;

    /// Get response time estimate in milliseconds
    fn estimated_response_time(&self) -> u64;

    /// Check if the adapter has rate limiting
    fn has_rate_limiting(&self) -> bool;
}

/// Jikan (MyAnimeList) specific mapper implementation
#[derive(Debug, Clone)]
pub struct JikanMapper;

impl JikanMapper {
    pub fn new() -> Self {
        Self
    }

    /// Map Jikan rating to age restriction
    fn map_rating_to_age_restriction(rating: &Option<String>) -> Option<UnifiedAgeRestriction> {
        log::info!("JIKAN RATING MAPPING: input = {:?}", rating);
        let result = rating.as_ref().and_then(|r| match r.as_str() {
            "G - All Ages" => Some(UnifiedAgeRestriction::GeneralAudiences),
            "PG - Children" => Some(UnifiedAgeRestriction::GeneralAudiences),
            "PG-13 - Teens 13 or older" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "R - 17+ (violence & profanity)" => Some(UnifiedAgeRestriction::ParentalGuidance17),
            "R+ - Mild Nudity" => Some(UnifiedAgeRestriction::Mature),
            "Rx - Hentai" => Some(UnifiedAgeRestriction::Explicit),
            _ => None,
        });
        log::info!("JIKAN RATING MAPPING: result = {:?}", result);
        result
    }

    /// Map Jikan status to AnimeStatus
    fn map_anime_status(status: &Option<String>) -> AnimeStatus {
        match status.as_ref().map(|s| s.as_str()) {
            Some("Finished Airing") => AnimeStatus::Finished,
            Some("Currently Airing") => AnimeStatus::Airing,
            Some("Not yet aired") => AnimeStatus::NotYetAired,
            _ => AnimeStatus::Unknown,
        }
    }

    /// Map Jikan anime type to AnimeType
    fn map_anime_type(anime_type: &Option<String>) -> AnimeType {
        match anime_type.as_ref().map(|s| s.as_str()) {
            Some("TV") => AnimeType::TV,
            Some("Movie") => AnimeType::Movie,
            Some("OVA") => AnimeType::OVA,
            Some("Special") => AnimeType::Special,
            Some("ONA") => AnimeType::ONA,
            Some("Music") => AnimeType::Music,
            _ => AnimeType::Unknown,
        }
    }

    /// Extract genres from MAL entities
    fn extract_genres(genres: &Option<Vec<MalEntity>>) -> Vec<Genre> {
        genres
            .as_ref()
            .map(|g| {
                g.iter()
                    .map(|entity| Genre::new(entity.name.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract studios from MAL entities
    fn extract_studios(studios: &Option<Vec<MalEntity>>) -> Vec<String> {
        studios
            .as_ref()
            .map(|s| s.iter().map(|entity| entity.name.clone()).collect())
            .unwrap_or_default()
    }

    /// Extract best image URL from Jikan images
    fn extract_image_url(images: &Option<Images>) -> Option<String> {
        images.as_ref().and_then(|img| {
            // Prefer larger images
            img.jpg.as_ref().and_then(|jpg| {
                jpg.large_image_url
                    .clone()
                    .or_else(|| jpg.image_url.clone())
                    .or_else(|| jpg.small_image_url.clone())
            })
        })
    }

    /// Extract trailer URL from Jikan trailer data
    fn extract_trailer_url(trailer: &Option<Trailer>) -> Option<String> {
        trailer.as_ref().and_then(|t| t.url.clone())
    }

    /// Parse Jikan aired dates
    fn parse_aired_date(aired: &Option<Aired>) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
        let from = aired
            .as_ref()
            .and_then(|a| a.from.as_ref())
            .and_then(|date_str| DateTime::parse_from_rfc3339(date_str).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let to = aired
            .as_ref()
            .and_then(|a| a.to.as_ref())
            .and_then(|date_str| DateTime::parse_from_rfc3339(date_str).ok())
            .map(|dt| dt.with_timezone(&Utc));

        (from, to)
    }

    /// Calculate data completeness based on available fields
    fn calculate_completeness(anime: &AnimeDetailed) -> f32 {
        let mut fields_present = 0;
        let mut total_fields = 0;

        // Check core fields
        total_fields += 12;
        if !anime.title.main.is_empty() {
            fields_present += 1;
        }
        if anime.title.english.is_some() {
            fields_present += 1;
        }
        if anime.title.japanese.is_some() {
            fields_present += 1;
        }
        if anime.synopsis.is_some() {
            fields_present += 1;
        }
        if anime.episodes.is_some() {
            fields_present += 1;
        }
        if anime.score.is_some() {
            fields_present += 1;
        }
        if anime.image_url.is_some() {
            fields_present += 1;
        }
        if !anime.genres.is_empty() {
            fields_present += 1;
        }
        if !anime.studios.is_empty() {
            fields_present += 1;
        }
        if anime.aired.from.is_some() {
            fields_present += 1;
        }
        if anime.age_restriction.is_some() {
            fields_present += 1;
        }
        if anime.source.is_some() {
            fields_present += 1;
        }

        fields_present as f32 / total_fields as f32
    }

    /// Identify missing critical fields
    fn identify_missing_fields(anime: &AnimeDetailed) -> Vec<String> {
        let mut missing = Vec::new();

        if anime.title.english.is_none() {
            missing.push("title_english".to_string());
        }
        if anime.synopsis.is_none() {
            missing.push("synopsis".to_string());
        }
        if anime.episodes.is_none() {
            missing.push("episodes".to_string());
        }
        if anime.image_url.is_none() {
            missing.push("cover_image".to_string());
        }
        if anime.genres.is_empty() {
            missing.push("genres".to_string());
        }
        if anime.banner_image.is_none() {
            missing.push("banner_image".to_string());
        }

        missing
    }
}

impl AnimeMapper<Anime> for JikanMapper {
    fn map_to_anime_data(&self, source: Anime) -> Result<AnimeData, AppError> {
        let now = Utc::now();

        // Create provider metadata
        let provider_metadata =
            ProviderMetadata::new(AnimeProvider::Jikan, source.mal_id.to_string());

        // Parse aired dates
        let (aired_from, aired_to) = Self::parse_aired_date(&source.aired);

        // Create the AnimeDetailed entity
        let anime_detailed = AnimeDetailed {
            id: Uuid::new_v4(),
            title: AnimeTitle {
                main: source
                    .title
                    .clone()
                    .unwrap_or_else(|| "Unknown Title".to_string()),
                english: source.title_english,
                japanese: source.title_japanese.clone(),
                romaji: source.title.clone(),
                native: source.title_japanese,
                synonyms: source.title_synonyms.unwrap_or_default(),
            },
            provider_metadata,
            score: source.score.map(|s| (s * 100.0).round() / 100.0), // Round to 2 decimal places
            rating: source.score.map(|s| (s * 100.0).round() / 100.0), // Round to 2 decimal places
            favorites: source.favorites.map(|f| f as u32),
            synopsis: source.synopsis.clone(),
            description: source.synopsis,
            episodes: source.episodes.map(|e| e as u16),
            status: Self::map_anime_status(&source.status),
            aired: AiredDates {
                from: aired_from,
                to: aired_to,
            },
            anime_type: Self::map_anime_type(&source.r#type),
            age_restriction: {
                let age_restriction = Self::map_rating_to_age_restriction(&source.rating);
                log::info!(
                    "JIKAN FINAL AGE_RESTRICTION for '{}': {:?}",
                    source.title.as_ref().unwrap_or(&"Unknown".to_string()),
                    age_restriction
                );
                age_restriction
            },
            genres: Self::extract_genres(&source.genres),
            studios: Self::extract_studios(&source.studios),
            source: source.source,
            duration: source.duration,
            image_url: Self::extract_image_url(&source.images),
            images: Self::extract_image_url(&source.images),
            banner_image: None, // Jikan doesn't provide banner images
            trailer_url: Self::extract_trailer_url(&source.trailer),
            composite_score: source
                .score
                .map(|s| (s * 100.0).round() / 100.0)
                .unwrap_or(0.0), // Round to 2 decimal places
            tier: AnimeTier::default(),
            quality_metrics: QualityMetrics::default(),
            created_at: now,
            updated_at: now,
            last_synced_at: Some(now),
        };

        // Create quality assessment
        let quality = DataQuality {
            score: 0.85, // High quality but less comprehensive than AniList
            completeness: Self::calculate_completeness(&anime_detailed),
            consistency: 0.95,    // MAL data is very consistent
            relevance_score: 0.0, // Will be set during search ranking
            missing_fields: Self::identify_missing_fields(&anime_detailed),
        };

        // Create source information
        let source_info = DataSource {
            primary_provider: AnimeProvider::Jikan,
            providers_used: vec![AnimeProvider::Jikan],
            confidence: 0.9,     // MAL is very reliable
            fetch_time_ms: 1200, // Typical Jikan response time
        };

        Ok(AnimeData::with_metadata(
            anime_detailed,
            quality,
            source_info,
        ))
    }
}

impl AdapterCapabilities for JikanMapper {
    fn name(&self) -> &'static str {
        "Jikan (MyAnimeList)"
    }

    fn supported_fields(&self) -> Vec<&'static str> {
        vec![
            "id",
            "mal_id",
            "title",
            "title_english",
            "title_japanese",
            "title_synonyms",
            "synopsis",
            "description",
            "episode_count",
            "duration",
            "status",
            "anime_type",
            "source",
            "start_date",
            "end_date",
            "season",
            "year",
            "cover_image",
            "trailer_url",
            "score",
            "popularity",
            "favorites",
            "mean_score",
            "age_restriction",
            "is_adult",
            "studios",
            "producers",
            "genres",
            "tags",
            "external_links",
            "streaming_links",
        ]
    }

    fn unsupported_fields(&self) -> Vec<&'static str> {
        vec![
            "anilist_id",   // Not available in Jikan
            "banner_image", // Not provided by MyAnimeList
        ]
    }

    fn quality_score(&self) -> f64 {
        0.85 // High quality but less comprehensive than AniList
    }

    fn estimated_response_time(&self) -> u64 {
        1200 // ~1.2s average, slower than AniList
    }

    fn has_rate_limiting(&self) -> bool {
        true // Jikan has rate limiting
    }
}

impl Default for JikanMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl JikanMapper {}
