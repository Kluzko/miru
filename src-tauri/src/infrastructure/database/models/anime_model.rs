use crate::infrastructure::database::schema::*;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = anime)]
pub struct AnimeModel {
    pub id: Uuid,
    pub mal_id: Option<i32>,
    pub title: String,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub score: Option<f32>,
    pub scored_by: Option<i32>,
    pub rank: Option<i32>,
    pub popularity: Option<i32>,
    pub members: Option<i32>,
    pub favorites: Option<i32>,
    pub synopsis: Option<String>,
    pub episodes: Option<i32>,
    pub status: String,
    pub aired_from: Option<DateTime<Utc>>,
    pub aired_to: Option<DateTime<Utc>>,
    pub anime_type: String,
    pub rating: Option<String>,
    pub source: Option<String>,
    pub duration: Option<String>,
    pub image_url: Option<String>,
    pub mal_url: Option<String>,
    pub composite_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = genres)]
pub struct GenreModel {
    pub id: Uuid,
    pub mal_id: Option<i32>,
    pub name: String,
}

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = anime_genres)]
pub struct AnimeGenreModel {
    pub anime_id: Uuid,
    pub genre_id: Uuid,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = collections)]
pub struct CollectionModel {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = collection_anime)]
pub struct CollectionAnimeModel {
    pub collection_id: Uuid,
    pub anime_id: Uuid,
    pub added_at: DateTime<Utc>,
    pub user_score: Option<f32>,
    pub notes: Option<String>,
}

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = studios)]
pub struct StudioModel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = anime_studios)]
pub struct AnimeStudioModel {
    pub anime_id: Uuid,
    pub studio_id: Uuid,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
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
