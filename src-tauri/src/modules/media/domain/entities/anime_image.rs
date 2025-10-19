use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::modules::media::domain::value_objects::{AnimeProvider, ImageType};
use crate::schema::anime_images;

/// Anime image entity from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, Type)]
#[diesel(table_name = anime_images)]
pub struct AnimeImage {
    pub id: Uuid,
    pub anime_id: Uuid,
    pub provider: AnimeProvider,
    pub provider_image_id: Option<String>,
    pub image_type: ImageType,
    pub is_primary: bool,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub aspect_ratio: Option<f32>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<i32>,
    pub language: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
}

/// New anime image for insertion
#[derive(Debug, Clone, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = anime_images)]
pub struct NewAnimeImage {
    pub anime_id: Uuid,
    pub provider: AnimeProvider,
    pub provider_image_id: Option<String>,
    pub image_type: ImageType,
    pub is_primary: bool,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<i32>,
    pub language: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub synced_at: Option<DateTime<Utc>>,
}

impl AnimeImage {
    /// Check if this image has quality metrics
    pub fn has_quality_metrics(&self) -> bool {
        self.vote_average.is_some() && self.vote_count.is_some()
    }

    /// Get quality score (0.0 to 10.0)
    pub fn quality_score(&self) -> Option<f32> {
        self.vote_average
    }

    /// Check if this is a high quality image (vote_average >= 7.0)
    pub fn is_high_quality(&self) -> bool {
        self.vote_average.map(|score| score >= 7.0).unwrap_or(false)
    }

    /// Get computed or typical aspect ratio
    pub fn get_aspect_ratio(&self) -> Option<f32> {
        self.aspect_ratio
            .or_else(|| self.image_type.typical_aspect_ratio())
    }

    /// Check if dimensions are available
    pub fn has_dimensions(&self) -> bool {
        self.width.is_some() && self.height.is_some()
    }
}

impl NewAnimeImage {
    /// Create a new anime image
    pub fn new(
        anime_id: Uuid,
        provider: AnimeProvider,
        image_type: ImageType,
        url: String,
    ) -> Self {
        Self {
            anime_id,
            provider,
            provider_image_id: None,
            image_type,
            is_primary: false,
            url,
            width: None,
            height: None,
            vote_average: None,
            vote_count: None,
            language: None,
            file_size_bytes: None,
            synced_at: Some(Utc::now()),
        }
    }

    /// Set as primary image
    pub fn with_primary(mut self, is_primary: bool) -> Self {
        self.is_primary = is_primary;
        self
    }

    /// Set provider image ID
    pub fn with_provider_id(mut self, provider_image_id: String) -> Self {
        self.provider_image_id = Some(provider_image_id);
        self
    }

    /// Set dimensions
    pub fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set quality metrics
    pub fn with_quality(mut self, vote_average: f32, vote_count: i32) -> Self {
        self.vote_average = Some(vote_average);
        self.vote_count = Some(vote_count);
        self
    }

    /// Set language
    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }
}
