use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::modules::media::domain::value_objects::{AnimeProvider, VideoType};
use crate::schema::anime_videos;

/// Anime video entity from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, Type)]
#[diesel(table_name = anime_videos)]
pub struct AnimeVideo {
    pub id: Uuid,
    pub anime_id: Uuid,
    pub provider: AnimeProvider,
    pub provider_video_id: Option<String>,
    pub video_type: VideoType,
    pub is_official: bool,
    pub name: String,
    pub site: String,
    pub key: String,
    pub url: String,
    pub resolution: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub language: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
}

/// New anime video for insertion
#[derive(Debug, Clone, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = anime_videos)]
pub struct NewAnimeVideo {
    pub anime_id: Uuid,
    pub provider: AnimeProvider,
    pub provider_video_id: Option<String>,
    pub video_type: VideoType,
    pub is_official: bool,
    pub name: String,
    pub site: String,
    pub key: String,
    pub url: String,
    pub resolution: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub language: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub synced_at: Option<DateTime<Utc>>,
}

impl AnimeVideo {
    /// Check if this is a YouTube video
    pub fn is_youtube(&self) -> bool {
        self.site.eq_ignore_ascii_case("youtube")
    }

    /// Get YouTube embed URL
    pub fn youtube_embed_url(&self) -> Option<String> {
        if self.is_youtube() {
            Some(format!("https://www.youtube.com/embed/{}", self.key))
        } else {
            None
        }
    }

    /// Check if this is HD quality (resolution >= 720)
    pub fn is_hd(&self) -> bool {
        self.resolution.map(|r| r >= 720).unwrap_or(false)
    }

    /// Check if this is Full HD quality (resolution >= 1080)
    pub fn is_full_hd(&self) -> bool {
        self.resolution.map(|r| r >= 1080).unwrap_or(false)
    }

    /// Get duration in minutes
    pub fn duration_minutes(&self) -> Option<i32> {
        self.duration_seconds.map(|s| s / 60)
    }

    /// Check if this is promotional content
    pub fn is_promotional(&self) -> bool {
        self.video_type.is_promotional()
    }

    /// Check if this is actual anime content
    pub fn is_content(&self) -> bool {
        self.video_type.is_content()
    }
}

impl NewAnimeVideo {
    /// Create a new anime video
    pub fn new(
        anime_id: Uuid,
        provider: AnimeProvider,
        video_type: VideoType,
        name: String,
        site: String,
        key: String,
        url: String,
    ) -> Self {
        Self {
            anime_id,
            provider,
            provider_video_id: None,
            video_type,
            is_official: false,
            name,
            site,
            key,
            url,
            resolution: None,
            duration_seconds: None,
            language: None,
            published_at: None,
            synced_at: Some(Utc::now()),
        }
    }

    /// Create a YouTube video
    pub fn youtube(
        anime_id: Uuid,
        provider: AnimeProvider,
        video_type: VideoType,
        name: String,
        youtube_key: String,
    ) -> Self {
        let url = format!("https://www.youtube.com/watch?v={}", youtube_key);
        Self::new(
            anime_id,
            provider,
            video_type,
            name,
            "YouTube".to_string(),
            youtube_key,
            url,
        )
    }

    /// Set as official video
    pub fn with_official(mut self, is_official: bool) -> Self {
        self.is_official = is_official;
        self
    }

    /// Set provider video ID
    pub fn with_provider_id(mut self, provider_video_id: String) -> Self {
        self.provider_video_id = Some(provider_video_id);
        self
    }

    /// Set resolution
    pub fn with_resolution(mut self, resolution: i32) -> Self {
        self.resolution = Some(resolution);
        self
    }

    /// Set duration
    pub fn with_duration_seconds(mut self, duration_seconds: i32) -> Self {
        self.duration_seconds = Some(duration_seconds);
        self
    }

    /// Set language
    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }

    /// Set published date
    pub fn with_published_at(mut self, published_at: DateTime<Utc>) -> Self {
        self.published_at = Some(published_at);
        self
    }
}
