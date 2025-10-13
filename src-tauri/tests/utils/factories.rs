/// Test data factories using builder pattern
///
/// Provides convenient methods to create test data with sensible defaults
use chrono::TimeZone;
use miru_lib::modules::{
    anime::domain::{
        entities::{anime_detailed::AiredDates, anime_detailed::AnimeDetailed, genre::Genre},
        value_objects::{
            anime_status::AnimeStatus, anime_tier::AnimeTier, anime_title::AnimeTitle,
            anime_type::AnimeType, quality_metrics::QualityMetrics,
        },
    },
    provider::{domain::AnimeProvider, ProviderMetadata},
};
use miru_lib::shared::domain::value_objects::UnifiedAgeRestriction;
use uuid::Uuid;

pub struct AnimeFactory {
    id: Uuid,
    title: String,
    provider: AnimeProvider,
    provider_id: String,
    score: Option<f32>,
    rating: Option<f32>,
    favorites: Option<u32>,
    synopsis: Option<String>,
    description: Option<String>,
    episodes: Option<u16>,
    status: AnimeStatus,
    aired_from: Option<chrono::DateTime<chrono::Utc>>,
    aired_to: Option<chrono::DateTime<chrono::Utc>>,
    anime_type: AnimeType,
    age_restriction: Option<UnifiedAgeRestriction>,
    genres: Vec<Genre>,
    studios: Vec<String>,
    source: Option<String>,
    duration: Option<String>,
    image_url: Option<String>,
    banner_image: Option<String>,
    trailer_url: Option<String>,
}

impl Default for AnimeFactory {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "Test Anime".to_string(),
            provider: AnimeProvider::AniList,
            provider_id: format!("{}", rand::random::<u32>() % 1000000 + 1000),
            score: None,
            rating: None,
            favorites: None,
            synopsis: None,
            description: None,
            episodes: None,
            status: AnimeStatus::Unknown,
            aired_from: None,
            aired_to: None,
            anime_type: AnimeType::Unknown,
            age_restriction: None,
            genres: Vec::new(),
            studios: Vec::new(),
            source: None,
            duration: None,
            image_url: None,
            banner_image: None,
            trailer_url: None,
        }
    }
}

impl AnimeFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn minimal() -> Self {
        Self::default()
    }

    pub fn complete() -> Self {
        Self::default()
            .with_title("Complete Test Anime")
            .with_score(8.5)
            .with_favorites(10000)
            .with_synopsis("A comprehensive test anime with full data")
            .with_episodes(24)
            .with_status(AnimeStatus::Finished)
            .with_aired_dates(
                chrono::Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
                chrono::Utc.with_ymd_and_hms(2020, 6, 30, 0, 0, 0).unwrap(),
            )
            .with_anime_type(AnimeType::TV)
            .with_age_restriction(UnifiedAgeRestriction::ParentalGuidance13)
            .with_genres(vec!["Action", "Adventure", "Fantasy"])
            .with_studios(vec!["Test Studio"])
            .with_source("Manga")
            .with_duration("24 min")
            .with_image("https://example.com/image.jpg")
            .with_banner("https://example.com/banner.jpg")
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_provider(mut self, provider: AnimeProvider, external_id: &str) -> Self {
        self.provider = provider;
        self.provider_id = external_id.to_string();
        self
    }

    pub fn with_anilist_id(self, id: u32) -> Self {
        self.with_provider(AnimeProvider::AniList, &id.to_string())
    }

    pub fn with_score(mut self, score: f32) -> Self {
        self.score = Some(score);
        self.rating = Some(score);
        self
    }

    pub fn with_favorites(mut self, favorites: u32) -> Self {
        self.favorites = Some(favorites);
        self
    }

    pub fn with_synopsis(mut self, synopsis: &str) -> Self {
        self.synopsis = Some(synopsis.to_string());
        self.description = Some(synopsis.to_string());
        self
    }

    pub fn with_episodes(mut self, episodes: u16) -> Self {
        self.episodes = Some(episodes);
        self
    }

    pub fn with_status(mut self, status: AnimeStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_aired_dates(
        mut self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.aired_from = Some(from);
        self.aired_to = Some(to);
        self
    }

    pub fn with_anime_type(mut self, anime_type: AnimeType) -> Self {
        self.anime_type = anime_type;
        self
    }

    pub fn with_age_restriction(mut self, age_restriction: UnifiedAgeRestriction) -> Self {
        self.age_restriction = Some(age_restriction);
        self
    }

    pub fn with_genres(mut self, genre_names: Vec<&str>) -> Self {
        self.genres = genre_names
            .into_iter()
            .map(|name| Genre {
                id: Uuid::new_v4(),
                name: name.to_string(),
            })
            .collect();
        self
    }

    pub fn with_studios(mut self, studios: Vec<&str>) -> Self {
        self.studios = studios.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }

    pub fn with_duration(mut self, duration: &str) -> Self {
        self.duration = Some(duration.to_string());
        self
    }

    pub fn with_image(mut self, url: &str) -> Self {
        self.image_url = Some(url.to_string());
        self
    }

    pub fn with_banner(mut self, url: &str) -> Self {
        self.banner_image = Some(url.to_string());
        self
    }

    pub fn build(self) -> AnimeDetailed {
        let now = chrono::Utc::now();

        // Calculate quality metrics based on data completeness
        let completeness = self.calculate_completeness();
        let quality_metrics = QualityMetrics {
            popularity_score: self.score.unwrap_or(5.0),
            engagement_score: (self.favorites.unwrap_or(0) as f32 / 1000.0).min(10.0),
            consistency_score: completeness,
            audience_reach_score: if self.genres.is_empty() { 5.0 } else { 8.0 },
        };

        // Calculate composite score (0-10 scale)
        let composite_score = (quality_metrics.popularity_score * 0.3
            + quality_metrics.engagement_score * 0.2
            + quality_metrics.consistency_score * 10.0 * 0.3
            + quality_metrics.audience_reach_score * 0.2)
            .min(10.0);

        // Determine tier based on score
        let tier = if composite_score >= 9.0 {
            AnimeTier::S
        } else if composite_score >= 8.0 {
            AnimeTier::A
        } else if composite_score >= 6.0 {
            AnimeTier::B
        } else if composite_score >= 4.0 {
            AnimeTier::C
        } else {
            AnimeTier::D
        };

        AnimeDetailed {
            id: self.id,
            title: AnimeTitle::new(self.title),
            provider_metadata: ProviderMetadata::new(self.provider, self.provider_id),
            score: self.score,
            rating: self.rating,
            favorites: self.favorites,
            synopsis: self.synopsis,
            description: self.description,
            episodes: self.episodes,
            status: self.status,
            aired: AiredDates {
                from: self.aired_from,
                to: self.aired_to,
            },
            anime_type: self.anime_type,
            age_restriction: self.age_restriction,
            genres: self.genres,
            studios: self.studios,
            source: self.source,
            duration: self.duration,
            image_url: self.image_url.clone(),
            images: self.image_url,
            banner_image: self.banner_image,
            trailer_url: self.trailer_url,
            composite_score,
            tier,
            quality_metrics,
            created_at: now,
            updated_at: now,
            last_synced_at: None,
        }
    }

    fn calculate_completeness(&self) -> f32 {
        let mut filled_fields = 0;
        let total_fields = 15;

        if self.score.is_some() {
            filled_fields += 1;
        }
        if self.synopsis.is_some() {
            filled_fields += 1;
        }
        if self.episodes.is_some() {
            filled_fields += 1;
        }
        if self.status != AnimeStatus::Unknown {
            filled_fields += 1;
        }
        if self.aired_from.is_some() {
            filled_fields += 1;
        }
        if self.anime_type != AnimeType::Unknown {
            filled_fields += 1;
        }
        if self.age_restriction.is_some() {
            filled_fields += 1;
        }
        if !self.genres.is_empty() {
            filled_fields += 1;
        }
        if !self.studios.is_empty() {
            filled_fields += 1;
        }
        if self.source.is_some() {
            filled_fields += 1;
        }
        if self.duration.is_some() {
            filled_fields += 1;
        }
        if self.image_url.is_some() {
            filled_fields += 1;
        }
        if self.banner_image.is_some() {
            filled_fields += 1;
        }
        if self.trailer_url.is_some() {
            filled_fields += 1;
        }
        if self.favorites.is_some() {
            filled_fields += 1;
        }

        filled_fields as f32 / total_fields as f32
    }
}

// Convenience functions
pub fn minimal_anime() -> AnimeDetailed {
    AnimeFactory::minimal().build()
}

pub fn complete_anime() -> AnimeDetailed {
    AnimeFactory::complete().build()
}

pub fn anime_with_anilist_id(id: u32) -> AnimeDetailed {
    AnimeFactory::minimal().with_anilist_id(id).build()
}
