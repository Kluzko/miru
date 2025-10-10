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
use chrono::{DateTime, NaiveDate, Utc};
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

/// AniList specific mapper implementation
#[derive(Debug, Clone)]
pub struct AniListMapper;

impl AniListMapper {
    pub fn new() -> Self {
        Self
    }

    /// Map AniList fuzzy date to DateTime
    fn map_fuzzy_date_to_datetime(date: &Option<FuzzyDate>) -> Option<DateTime<Utc>> {
        date.as_ref().and_then(|d| {
            if let (Some(year), Some(month), Some(day)) = (d.year, d.month, d.day) {
                NaiveDate::from_ymd_opt(year, month as u32, day as u32)
                    .and_then(|date| date.and_hms_opt(0, 0, 0))
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            } else if let (Some(year), Some(month)) = (d.year, d.month) {
                NaiveDate::from_ymd_opt(year, month as u32, 1)
                    .and_then(|date| date.and_hms_opt(0, 0, 0))
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            } else if let Some(year) = d.year {
                NaiveDate::from_ymd_opt(year, 1, 1)
                    .and_then(|date| date.and_hms_opt(0, 0, 0))
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
            } else {
                None
            }
        })
    }

    /// Map AniList media status to AnimeStatus
    fn map_anime_status(status: &Option<MediaStatus>) -> AnimeStatus {
        match status {
            Some(MediaStatus::Finished) => AnimeStatus::Finished,
            Some(MediaStatus::Releasing) => AnimeStatus::Airing,
            Some(MediaStatus::NotYetReleased) => AnimeStatus::NotYetAired,
            Some(MediaStatus::Cancelled) => AnimeStatus::Cancelled,
            Some(MediaStatus::Hiatus) => AnimeStatus::Cancelled, // Map hiatus to cancelled
            _ => AnimeStatus::Unknown,
        }
    }

    /// Map AniList media format to AnimeType
    fn map_anime_type(format: &Option<MediaFormat>) -> AnimeType {
        match format {
            Some(MediaFormat::Tv) => AnimeType::TV,
            Some(MediaFormat::TvShort) => AnimeType::TV,
            Some(MediaFormat::Movie) => AnimeType::Movie,
            Some(MediaFormat::Special) => AnimeType::Special,
            Some(MediaFormat::Ova) => AnimeType::OVA,
            Some(MediaFormat::Ona) => AnimeType::ONA,
            Some(MediaFormat::Music) => AnimeType::Music,
            _ => AnimeType::Unknown,
        }
    }

    /// Map adult content flag to age restriction
    fn map_age_restriction(is_adult: Option<bool>) -> Option<UnifiedAgeRestriction> {
        match is_adult {
            Some(true) => Some(UnifiedAgeRestriction::Explicit),
            _ => None,
        }
    }

    /// Extract genres from genre list
    fn extract_genres(genres: &Option<Vec<String>>) -> Vec<Genre> {
        genres
            .as_ref()
            .map(|genres| genres.iter().map(|name| Genre::new(name.clone())).collect())
            .unwrap_or_default()
    }

    /// Extract studios from studio connection
    /// Handles both nodes (search queries) and edges (detail queries) structures
    /// Prioritizes main studios but includes all if no main studios are marked
    fn extract_studios(studios: &Option<StudioConnection>) -> Vec<String> {
        let connection = match studios.as_ref() {
            Some(conn) => conn,
            None => return Vec::new(),
        };

        // Try edges first (detail queries with isMain on edge)
        if let Some(edges) = &connection.edges {
            // First try to get only main studios
            let main_studios: Vec<String> = edges
                .iter()
                .filter(|edge| edge.is_main.unwrap_or(false))
                .filter_map(|edge| edge.node.as_ref())
                .filter_map(|studio| studio.name.clone())
                .collect();

            // If we found main studios, return them
            if !main_studios.is_empty() {
                return main_studios;
            }

            // Otherwise, return all studios (fallback when isMain is not set)
            return edges
                .iter()
                .filter_map(|edge| edge.node.as_ref())
                .filter_map(|studio| studio.name.clone())
                .collect();
        }

        // Fallback to nodes (search queries with isMain on node)
        if let Some(nodes) = &connection.nodes {
            // First try to get only main studios
            let main_studios: Vec<String> = nodes
                .iter()
                .filter(|studio| studio.is_main.unwrap_or(false))
                .filter_map(|studio| studio.name.clone())
                .collect();

            // If we found main studios, return them
            if !main_studios.is_empty() {
                return main_studios;
            }

            // Otherwise, return all studios (fallback when isMain is not set)
            return nodes
                .iter()
                .filter_map(|studio| studio.name.clone())
                .collect();
        }

        Vec::new()
    }

    /// Extract cover image URL
    fn extract_cover_image(cover_image: &Option<MediaCoverImage>) -> Option<String> {
        cover_image.as_ref().and_then(|img| {
            img.extra_large
                .clone()
                .or_else(|| img.large.clone())
                .or_else(|| img.medium.clone())
        })
    }

    /// Extract trailer URL
    fn extract_trailer_url(trailer: &Option<MediaTrailer>) -> Option<String> {
        trailer.as_ref().and_then(|t| {
            if let (Some(site), Some(id)) = (&t.site, &t.id) {
                match site.to_lowercase().as_str() {
                    "youtube" => Some(format!("https://www.youtube.com/watch?v={}", id)),
                    "dailymotion" => Some(format!("https://www.dailymotion.com/video/{}", id)),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    /// Calculate data completeness based on available fields
    fn calculate_completeness(anime: &AnimeDetailed) -> f32 {
        let mut fields_present = 0;
        let mut total_fields = 0;

        // Check core fields
        total_fields += 10;
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
        if anime.age_restriction.is_none() {
            missing.push("age_restriction".to_string());
        }

        missing
    }
}

impl AnimeMapper<Media> for AniListMapper {
    fn map_to_anime_data(&self, source: Media) -> Result<AnimeData, AppError> {
        let now = Utc::now();

        // Create provider metadata
        let mut provider_metadata = ProviderMetadata::new(
            AnimeProvider::AniList,
            source.id.map(|id| id.to_string()).unwrap_or_default(),
        );

        // Add MAL ID if available
        if let Some(mal_id) = source.id_mal {
            provider_metadata.add_external_id(AnimeProvider::Jikan, mal_id.to_string());
        }

        // Create the AnimeDetailed entity
        let anime_detailed = AnimeDetailed {
            id: Uuid::new_v4(),
            title: AnimeTitle {
                main: source
                    .title
                    .as_ref()
                    .and_then(|t| {
                        t.romaji
                            .as_ref()
                            .or(t.english.as_ref())
                            .or(t.native.as_ref())
                    })
                    .cloned()
                    .unwrap_or_else(|| "Unknown Title".to_string()),
                english: source.title.as_ref().and_then(|t| t.english.clone()),
                japanese: source.title.as_ref().and_then(|t| t.native.clone()),
                romaji: source.title.as_ref().and_then(|t| t.romaji.clone()),
                native: source.title.as_ref().and_then(|t| t.native.clone()),
                synonyms: source.synonyms.unwrap_or_default(),
            },
            provider_metadata,
            score: source.average_score.map(|s| {
                let score = s as f32 / 10.0;
                (score * 100.0).round() / 100.0 // Round to 2 decimal places
            }),
            rating: source.average_score.map(|s| {
                let score = s as f32 / 10.0;
                (score * 100.0).round() / 100.0 // Round to 2 decimal places
            }),
            favorites: source.favourites.map(|f| f as u32),
            synopsis: source.description.clone(),
            description: source.description,
            episodes: source.episodes.map(|e| e as u16),
            status: Self::map_anime_status(&source.status),
            aired: AiredDates {
                from: Self::map_fuzzy_date_to_datetime(&source.start_date),
                to: Self::map_fuzzy_date_to_datetime(&source.end_date),
            },
            anime_type: Self::map_anime_type(&source.format),
            age_restriction: Self::map_age_restriction(source.is_adult),
            genres: Self::extract_genres(&source.genres),
            studios: Self::extract_studios(&source.studios),
            source: source.source.clone(),
            duration: source.duration.map(|d| format!("{} minutes", d)),
            image_url: Self::extract_cover_image(&source.cover_image),
            images: Self::extract_cover_image(&source.cover_image),
            banner_image: source.banner_image,
            trailer_url: Self::extract_trailer_url(&source.trailer),
            composite_score: source
                .average_score
                .map(|s| {
                    let score = s as f32 / 10.0;
                    (score * 100.0).round() / 100.0 // Round to 2 decimal places
                })
                .unwrap_or(0.0),
            tier: AnimeTier::default(),
            quality_metrics: QualityMetrics::default(),
            created_at: now,
            updated_at: now,
            last_synced_at: Some(now),
        };

        // Create quality assessment
        let quality = DataQuality {
            score: 0.95, // AniList generally has high quality data
            completeness: Self::calculate_completeness(&anime_detailed),
            consistency: 0.9,
            relevance_score: 0.0, // Will be set during search ranking
            missing_fields: Self::identify_missing_fields(&anime_detailed),
        };

        // Create source information
        let source_info = DataSource {
            primary_provider: AnimeProvider::AniList,
            providers_used: vec![AnimeProvider::AniList],
            confidence: 0.95,
            fetch_time_ms: 800, // Typical AniList response time
        };

        Ok(AnimeData::with_metadata(
            anime_detailed,
            quality,
            source_info,
        ))
    }
}

impl AdapterCapabilities for AniListMapper {
    fn name(&self) -> &'static str {
        "AniList"
    }

    fn supported_fields(&self) -> Vec<&'static str> {
        vec![
            "id",
            "anilist_id",
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
            "start_date",
            "end_date",
            "season",
            "year",
            "cover_image",
            "banner_image",
            "trailer_url",
            "score",
            "mean_score",
            "popularity",
            "favorites",
            "is_adult",
            "studios",
            "genres",
            "tags",
            "external_links",
            "streaming_links",
        ]
    }

    fn unsupported_fields(&self) -> Vec<&'static str> {
        vec![
            "age_restriction", // Not directly available (only adult flag)
            "source",          // Not in standard Media query
            "producers",       // Not clearly distinguished from studios
        ]
    }

    fn quality_score(&self) -> f64 {
        0.95 // Very high quality data
    }

    fn estimated_response_time(&self) -> u64 {
        800 // ~800ms average
    }

    fn has_rate_limiting(&self) -> bool {
        true // AniList has rate limiting
    }
}

impl Default for AniListMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anilist_mapper_capabilities() {
        let mapper = AniListMapper::new();

        assert_eq!(mapper.name(), "AniList");
        assert!(mapper.supports_field("title"));
        assert!(mapper.supports_field("cover_image"));
        assert!(!mapper.supports_field("age_restriction"));
        assert!(mapper.quality_score() > 0.9);
        assert!(mapper.has_rate_limiting());
    }

    #[test]
    fn test_map_anime_status() {
        assert_eq!(
            AniListMapper::map_anime_status(&Some(MediaStatus::Finished)),
            AnimeStatus::Finished
        );
        assert_eq!(
            AniListMapper::map_anime_status(&Some(MediaStatus::Releasing)),
            AnimeStatus::Airing
        );
        assert_eq!(AniListMapper::map_anime_status(&None), AnimeStatus::Unknown);
    }

    #[test]
    fn test_map_anime_type() {
        assert_eq!(
            AniListMapper::map_anime_type(&Some(MediaFormat::Tv)),
            AnimeType::TV
        );
        assert_eq!(
            AniListMapper::map_anime_type(&Some(MediaFormat::Movie)),
            AnimeType::Movie
        );
        assert_eq!(AniListMapper::map_anime_type(&None), AnimeType::Unknown);
    }
}

impl AniListMapper {}
