use super::models::{ContentRating, Genre as TmdbGenre, ProductionCompany, TvShow, TvShowDetails};
use crate::modules::anime::domain::{
    entities::{
        anime_detailed::{AiredDates, AnimeDetailed},
        genre::Genre,
    },
    value_objects::{AnimeStatus, AnimeTier, AnimeTitle, AnimeType, QualityMetrics},
};
use crate::modules::provider::domain::entities::anime_data::{AnimeData, DataQuality, DataSource};
use crate::shared::domain::value_objects::UnifiedAgeRestriction;
use crate::shared::domain::value_objects::{AnimeProvider, ProviderMetadata};
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

/// TMDB (The Movie Database) specific mapper implementation
#[derive(Debug, Clone)]
pub struct TmdbMapper;

impl TmdbMapper {
    pub fn new() -> Self {
        Self
    }

    /// Map TMDB content rating to age restriction
    fn map_rating_to_age_restriction(
        ratings: &Option<Vec<ContentRating>>,
    ) -> Option<UnifiedAgeRestriction> {
        // Prefer US ratings, fall back to JP ratings
        ratings.as_ref().and_then(|r| {
            r.iter()
                .find(|rating| rating.iso_3166_1 == "US")
                .or_else(|| r.iter().find(|rating| rating.iso_3166_1 == "JP"))
                .and_then(|rating| match rating.rating.as_str() {
                    "TV-Y" | "TV-G" | "G" => Some(UnifiedAgeRestriction::GeneralAudiences),
                    "TV-PG" | "PG" => Some(UnifiedAgeRestriction::GeneralAudiences),
                    "TV-14" | "PG-13" => Some(UnifiedAgeRestriction::ParentalGuidance13),
                    "TV-MA" | "R" | "NC-17" => Some(UnifiedAgeRestriction::Mature),
                    _ => None,
                })
        })
    }

    /// Map TMDB status to AnimeStatus
    fn map_anime_status(status: &Option<String>) -> AnimeStatus {
        match status.as_ref().map(|s| s.as_str()) {
            Some("Ended") | Some("Canceled") => AnimeStatus::Finished,
            Some("Returning Series") => AnimeStatus::Airing,
            Some("In Production") => AnimeStatus::NotYetAired,
            _ => AnimeStatus::Unknown,
        }
    }

    /// Map TMDB type to AnimeType
    fn map_anime_type(_tv_type: &Option<String>) -> AnimeType {
        // TMDB doesn't differentiate anime types well, default to TV
        AnimeType::TV
    }

    /// Extract genres from TMDB genres
    fn extract_genres(genres: &Option<Vec<TmdbGenre>>) -> Vec<Genre> {
        genres
            .as_ref()
            .map(|g| {
                g.iter()
                    .map(|entity| Genre::new(entity.name.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract studios/production companies
    fn extract_studios(companies: &Option<Vec<ProductionCompany>>) -> Vec<String> {
        companies
            .as_ref()
            .map(|s| s.iter().map(|entity| entity.name.clone()).collect())
            .unwrap_or_default()
    }

    /// Extract poster URL from TMDB poster path
    fn extract_poster_url(poster_path: &Option<String>) -> Option<String> {
        poster_path
            .as_ref()
            .map(|path| format!("https://image.tmdb.org/t/p/original{}", path))
    }

    /// Extract backdrop URL from TMDB backdrop path
    fn extract_backdrop_url(backdrop_path: &Option<String>) -> Option<String> {
        backdrop_path
            .as_ref()
            .map(|path| format!("https://image.tmdb.org/t/p/original{}", path))
    }

    /// Parse TMDB date string to DateTime
    fn parse_date(date_str: &Option<String>) -> Option<DateTime<Utc>> {
        date_str
            .as_ref()
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .and_then(|date| {
                date.and_hms_opt(0, 0, 0)
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
            })
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
        if anime.banner_image.is_some() {
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

    /// Map TvShow (search result) to AnimeData
    pub fn map_to_anime_data(&self, source: TvShow) -> Result<AnimeData, AppError> {
        let now = Utc::now();

        // Create provider metadata
        let provider_metadata = ProviderMetadata::new(AnimeProvider::TMDB, source.id.to_string());

        // Parse aired date
        let aired_from = Self::parse_date(&source.first_air_date);

        // Create the AnimeDetailed entity
        let anime_detailed = AnimeDetailed {
            id: Uuid::new_v4(),
            title: AnimeTitle {
                main: source
                    .name
                    .clone()
                    .unwrap_or_else(|| "Unknown Title".to_string()),
                english: source.name.clone(),
                japanese: source.original_name.clone(),
                romaji: source.name.clone(),
                native: source.original_name,
                synonyms: vec![],
            },
            provider_metadata,
            score: source.vote_average,
            rating: source.vote_average,
            favorites: None,
            synopsis: source.overview.clone(),
            description: source.overview,
            episodes: None, // Not available in search results
            status: AnimeStatus::Unknown,
            aired: AiredDates {
                from: aired_from,
                to: None,
            },
            anime_type: AnimeType::TV,
            age_restriction: None, // Not available in search results
            genres: vec![],        // Only genre IDs in search, need separate lookup
            studios: vec![],
            source: None,
            duration: None,
            image_url: Self::extract_poster_url(&source.poster_path),
            images: Self::extract_poster_url(&source.poster_path),
            banner_image: Self::extract_backdrop_url(&source.backdrop_path),
            trailer_url: None,
            composite_score: source.vote_average.unwrap_or(0.0),
            tier: AnimeTier::default(),
            quality_metrics: QualityMetrics::default(),
            created_at: now,
            updated_at: now,
            last_synced_at: Some(now),
        };

        // Create quality assessment
        let quality = DataQuality {
            score: 0.70, // TMDB has good image/video data but less anime-specific info
            completeness: Self::calculate_completeness(&anime_detailed),
            consistency: 0.90,
            relevance_score: 0.0,
            missing_fields: Self::identify_missing_fields(&anime_detailed),
        };

        // Create source information
        let source_info = DataSource {
            primary_provider: AnimeProvider::TMDB,
            providers_used: vec![AnimeProvider::TMDB],
            confidence: 0.75,
            fetch_time_ms: 300, // TMDB is fast
        };

        Ok(AnimeData::with_metadata(
            anime_detailed,
            quality,
            source_info,
        ))
    }

    /// Map TvShowDetails to AnimeData
    pub fn map_details_to_anime_data(&self, source: TvShowDetails) -> Result<AnimeData, AppError> {
        let now = Utc::now();

        // Create provider metadata
        let provider_metadata = ProviderMetadata::new(AnimeProvider::TMDB, source.id.to_string());

        // Parse aired dates
        let aired_from = Self::parse_date(&source.first_air_date);
        let aired_to = Self::parse_date(&source.last_air_date);

        // Extract average episode duration
        let duration = source
            .episode_run_time
            .as_ref()
            .and_then(|durations| durations.first())
            .map(|&minutes| format!("{} min", minutes));

        // Create the AnimeDetailed entity
        let anime_detailed = AnimeDetailed {
            id: Uuid::new_v4(),
            title: AnimeTitle {
                main: source
                    .name
                    .clone()
                    .unwrap_or_else(|| "Unknown Title".to_string()),
                english: source.name.clone(),
                japanese: source.original_name.clone(),
                romaji: source.name.clone(),
                native: source.original_name,
                synonyms: vec![],
            },
            provider_metadata,
            score: source.vote_average,
            rating: source.vote_average,
            favorites: None,
            synopsis: source.overview.clone(),
            description: source.overview,
            episodes: source.number_of_episodes.map(|e| e as u16),
            status: Self::map_anime_status(&source.status),
            aired: AiredDates {
                from: aired_from,
                to: aired_to,
            },
            anime_type: Self::map_anime_type(&source.r#type),
            age_restriction: None, // Need separate content_ratings API call
            genres: Self::extract_genres(&source.genres),
            studios: Self::extract_studios(&source.production_companies),
            source: None,
            duration,
            image_url: Self::extract_poster_url(&source.poster_path),
            images: Self::extract_poster_url(&source.poster_path),
            banner_image: Self::extract_backdrop_url(&source.backdrop_path),
            trailer_url: None, // Need separate videos API call
            composite_score: source.vote_average.unwrap_or(0.0),
            tier: AnimeTier::default(),
            quality_metrics: QualityMetrics::default(),
            created_at: now,
            updated_at: now,
            last_synced_at: Some(now),
        };

        // Create quality assessment
        let quality = DataQuality {
            score: 0.75, // Better quality with full details
            completeness: Self::calculate_completeness(&anime_detailed),
            consistency: 0.90,
            relevance_score: 0.0,
            missing_fields: Self::identify_missing_fields(&anime_detailed),
        };

        // Create source information
        let source_info = DataSource {
            primary_provider: AnimeProvider::TMDB,
            providers_used: vec![AnimeProvider::TMDB],
            confidence: 0.80,
            fetch_time_ms: 350,
        };

        Ok(AnimeData::with_metadata(
            anime_detailed,
            quality,
            source_info,
        ))
    }
}

impl Default for TmdbMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterCapabilities for TmdbMapper {
    fn name(&self) -> &'static str {
        "TMDB (The Movie Database)"
    }

    fn supported_fields(&self) -> Vec<&'static str> {
        vec![
            "id",
            "tmdb_id",
            "title",
            "title_english",
            "title_japanese",
            "synopsis",
            "description",
            "episode_count",
            "duration",
            "status",
            "anime_type",
            "start_date",
            "end_date",
            "cover_image",
            "banner_image",
            "posters",   // EXCELLENT categorization
            "backdrops", // EXCELLENT categorization
            "logos",     // EXCELLENT categorization
            "videos",    // EXCELLENT categorization
            "trailers",  // Well categorized
            "teasers",   // Well categorized
            "clips",     // Well categorized
            "score",
            "popularity",
            "genres",
            "studios",
            "production_companies",
            "networks",
            "external_links",
            "external_ids",    // IMDb, TVDB, etc.
            "content_ratings", // Age ratings by country
        ]
    }

    fn unsupported_fields(&self) -> Vec<&'static str> {
        vec![
            "mal_id",          // Not available
            "anilist_id",      // Not available
            "characters",      // Not available
            "staff",           // Limited
            "relations",       // Not available
            "tags",            // Limited
            "streaming_links", // Not available
        ]
    }

    fn quality_score(&self) -> f64 {
        0.75 // Good for images/videos, less comprehensive for anime-specific data
    }

    fn estimated_response_time(&self) -> u64 {
        300 // ~300ms average, very fast
    }

    fn has_rate_limiting(&self) -> bool {
        true // 50 requests/second
    }
}

// =============================================================================
// MEDIA MAPPING (Images & Videos)
// =============================================================================

use super::models::{Image, Video};
use crate::modules::media::domain::entities::{NewAnimeImage, NewAnimeVideo};
use crate::modules::media::domain::value_objects::ImageType;
use crate::modules::media::domain::value_objects::VideoType;

impl TmdbMapper {
    // =========================================================================
    // IMAGE MAPPING
    // =========================================================================

    /// Map TMDB Image to NewAnimeImage with specific type
    pub fn map_image(
        &self,
        tmdb_image: Image,
        anime_id: Uuid,
        image_type: ImageType,
        is_primary: bool,
    ) -> NewAnimeImage {
        let url = Self::build_image_url(&tmdb_image.file_path, "original");

        NewAnimeImage {
            anime_id,
            provider: AnimeProvider::TMDB,
            provider_image_id: Some(tmdb_image.file_path.clone()),
            image_type,
            is_primary,
            url,
            width: Some(tmdb_image.width as i32),
            height: Some(tmdb_image.height as i32),
            vote_average: Some(tmdb_image.vote_average),
            vote_count: Some(tmdb_image.vote_count as i32),
            language: tmdb_image.iso_639_1,
            file_size_bytes: None,
            synced_at: Some(Utc::now()),
        }
    }

    /// Map multiple TMDB images of a specific type
    pub fn map_images(
        &self,
        tmdb_images: Vec<Image>,
        anime_id: Uuid,
        image_type: ImageType,
    ) -> Vec<NewAnimeImage> {
        tmdb_images
            .into_iter()
            .enumerate()
            .map(|(idx, img)| {
                // Mark the first (highest rated) image as primary
                let is_primary = idx == 0;
                self.map_image(img, anime_id, image_type, is_primary)
            })
            .collect()
    }

    /// Build full TMDB image URL
    fn build_image_url(file_path: &str, size: &str) -> String {
        format!("https://image.tmdb.org/t/p/{}{}", size, file_path)
    }

    // =========================================================================
    // VIDEO MAPPING
    // =========================================================================

    /// Map TMDB Video to NewAnimeVideo
    pub fn map_video(&self, tmdb_video: Video, anime_id: Uuid) -> NewAnimeVideo {
        let video_type = Self::map_video_type(&tmdb_video.r#type);
        let url = Self::build_video_url(&tmdb_video.site, &tmdb_video.key);
        let published_at = Self::parse_published_at(&tmdb_video.published_at);

        NewAnimeVideo {
            anime_id,
            provider: AnimeProvider::TMDB,
            provider_video_id: Some(tmdb_video.id.clone()),
            video_type,
            is_official: tmdb_video.official,
            name: tmdb_video.name,
            site: tmdb_video.site,
            key: tmdb_video.key,
            url,
            resolution: Some(tmdb_video.size as i32),
            duration_seconds: None, // TMDB doesn't provide duration
            language: tmdb_video.iso_639_1,
            published_at,
            synced_at: Some(Utc::now()),
        }
    }

    /// Map multiple TMDB videos
    pub fn map_videos(&self, tmdb_videos: Vec<Video>, anime_id: Uuid) -> Vec<NewAnimeVideo> {
        tmdb_videos
            .into_iter()
            .map(|video| self.map_video(video, anime_id))
            .collect()
    }

    /// Build video URL based on site
    fn build_video_url(site: &str, key: &str) -> String {
        match site {
            "YouTube" => format!("https://www.youtube.com/watch?v={}", key),
            "Vimeo" => format!("https://vimeo.com/{}", key),
            _ => format!("{}:{}", site, key),
        }
    }

    /// Map TMDB video type string to our VideoType enum
    fn map_video_type(tmdb_type: &str) -> VideoType {
        match tmdb_type {
            "Trailer" => VideoType::Trailer,
            "Teaser" => VideoType::Teaser,
            "Clip" => VideoType::Clip,
            "Featurette" => VideoType::Featurette,
            "Behind the Scenes" => VideoType::BehindTheScenes,
            "Opening Credits" => VideoType::Opening,
            _ => VideoType::Clip, // Default to clip for unknown types
        }
    }

    /// Parse TMDB published_at timestamp
    fn parse_published_at(published_at: &str) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(published_at)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }
}
