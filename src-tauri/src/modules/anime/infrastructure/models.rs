use crate::modules::anime::{AnimeStatus, AnimeTier, AnimeType};
use crate::schema::{anime, anime_genres, anime_studios, genres, quality_metrics, studios};
use crate::shared::domain::value_objects::UnifiedAgeRestriction;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ================== ANIME MODELS ==================

// Optimized Selectable structs to avoid Diesel's 64-column-tables feature
// These allow selecting fewer columns for common queries

/// Lightweight anime selection for lists and searches
#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = anime)]
pub struct AnimeLite {
    pub id: Uuid,
    pub title_main: String,
    pub title_english: Option<String>,
    pub score: Option<f32>,
    pub episodes: Option<i32>,
    pub status: AnimeStatus,
    pub anime_type: AnimeType,
    pub image_url: Option<String>,
    pub composite_score: f32,
    pub tier: AnimeTier,
}

/// Medium anime selection with more details but not all fields
#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = anime)]
pub struct AnimeMedium {
    pub id: Uuid,
    pub title_main: String,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub title_romaji: Option<String>,
    pub title_native: Option<String>,
    pub score: Option<f32>,
    pub favorites: Option<i32>,
    pub synopsis: Option<String>,
    pub episodes: Option<i32>,
    pub status: AnimeStatus,
    pub anime_type: AnimeType,
    pub image_url: Option<String>,
    pub composite_score: f32,
    pub tier: AnimeTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// Main anime database model
#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = anime)]
pub struct Anime {
    pub id: Uuid,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub score: Option<f32>,
    pub favorites: Option<i32>,
    pub synopsis: Option<String>,
    pub episodes: Option<i32>,
    pub aired_from: Option<DateTime<Utc>>,
    pub aired_to: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub duration: Option<String>,
    pub image_url: Option<String>,
    pub composite_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title_main: String,
    pub title_romaji: Option<String>,
    pub title_native: Option<String>,
    pub title_synonyms: Option<serde_json::Value>,
    pub banner_image: Option<String>,
    pub trailer_url: Option<String>,
    pub tier: AnimeTier,
    pub quality_metrics: Option<serde_json::Value>,
    pub status: AnimeStatus,
    pub anime_type: AnimeType,
    pub age_restriction: Option<UnifiedAgeRestriction>,
    pub last_synced_at: Option<DateTime<Utc>>, // Must be last to match schema
}

/// Insert payload (write)
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = anime)]
pub struct NewAnime {
    pub id: Uuid,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub score: Option<f32>,
    pub favorites: Option<i32>,
    pub synopsis: Option<String>,
    pub episodes: Option<i32>,
    pub aired_from: Option<DateTime<Utc>>,
    pub aired_to: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub duration: Option<String>,
    pub image_url: Option<String>,
    pub composite_score: f32,
    pub title_main: String,
    pub title_romaji: Option<String>,
    pub title_native: Option<String>,
    pub title_synonyms: Option<serde_json::Value>,
    pub banner_image: Option<String>,
    pub trailer_url: Option<String>,
    pub tier: AnimeTier,
    pub quality_metrics: Option<serde_json::Value>,
    pub status: AnimeStatus,
    pub anime_type: AnimeType,
    pub age_restriction: Option<UnifiedAgeRestriction>,
    pub last_synced_at: Option<DateTime<Utc>>, // Must be last to match schema
}

/// Update payload (write) — excludes `id` and `created_at`
#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = anime)]
pub struct AnimeChangeset {
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub score: Option<f32>,
    pub favorites: Option<i32>,
    pub synopsis: Option<String>,
    pub episodes: Option<i32>,
    pub aired_from: Option<DateTime<Utc>>,
    pub aired_to: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub duration: Option<String>,
    pub image_url: Option<String>,
    pub composite_score: f32,
    pub updated_at: DateTime<Utc>,
    pub title_main: String,
    pub title_romaji: Option<String>,
    pub title_native: Option<String>,
    pub title_synonyms: Option<serde_json::Value>,
    pub banner_image: Option<String>,
    pub trailer_url: Option<String>,
    pub tier: AnimeTier,
    pub quality_metrics: Option<serde_json::Value>,
    pub status: AnimeStatus,
    pub anime_type: AnimeType,
    pub age_restriction: Option<UnifiedAgeRestriction>,
    pub last_synced_at: Option<DateTime<Utc>>, // Must be last to match schema
}

// ================== GENRE MODELS ==================

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = genres)]
pub struct GenreModel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = genres)]
pub struct NewGenre {
    pub id: Uuid,
    pub name: String,
}

// ============= ANIME-GENRE ASSOCIATION (join) =============

#[derive(Queryable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(Anime, foreign_key = anime_id))]
#[diesel(belongs_to(GenreModel, foreign_key = genre_id))]
#[diesel(table_name = anime_genres)]
#[diesel(primary_key(anime_id, genre_id))]
pub struct AnimeGenre {
    pub anime_id: Uuid,
    pub genre_id: Uuid,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = anime_genres)]
pub struct NewAnimeGenre {
    pub anime_id: Uuid,
    pub genre_id: Uuid,
}

// ================== STUDIO MODELS ==================

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = studios)]
pub struct StudioModel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = studios)]
pub struct NewStudio {
    pub id: Uuid,
    pub name: String,
}

// ============= ANIME-STUDIO ASSOCIATION (join) =============

#[derive(Queryable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(Anime, foreign_key = anime_id))]
#[diesel(belongs_to(StudioModel, foreign_key = studio_id))]
#[diesel(table_name = anime_studios)]
#[diesel(primary_key(anime_id, studio_id))]
pub struct AnimeStudio {
    pub anime_id: Uuid,
    pub studio_id: Uuid,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = anime_studios)]
pub struct NewAnimeStudio {
    pub anime_id: Uuid,
    pub studio_id: Uuid,
}

// ================== QUALITY METRICS MODELS ==================

#[derive(Queryable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(Anime, foreign_key = anime_id))]
#[diesel(table_name = quality_metrics)]
pub struct QualityMetricsModel {
    pub id: Uuid,
    pub anime_id: Uuid,
    pub popularity_score: f32,
    pub engagement_score: f32,
    pub consistency_score: f32,
    pub audience_reach_score: f32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = quality_metrics)]
pub struct NewQualityMetrics {
    pub id: Uuid,
    pub anime_id: Uuid,
    pub popularity_score: f32,
    pub engagement_score: f32,
    pub consistency_score: f32,
    pub audience_reach_score: f32,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = quality_metrics)]
pub struct QualityMetricsChangeset {
    pub popularity_score: f32,
    pub engagement_score: f32,
    pub consistency_score: f32,
    pub audience_reach_score: f32,
    pub updated_at: DateTime<Utc>,
}
