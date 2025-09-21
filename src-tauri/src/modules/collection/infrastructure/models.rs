use crate::schema::{collection_anime, collections};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============= COLLECTION MODELS =============

// For reading from database - with associations support
#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = collections)]
pub struct CollectionModel {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: Option<String>,
    pub is_public: Option<bool>,
}

// For inserting new collections
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = collections)]
pub struct NewCollection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

// For updating existing collections (excludes id and created_at)
#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = collections)]
pub struct CollectionChangeset {
    pub name: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

// ============= COLLECTION-ANIME ASSOCIATION =============

// For reading with associations
#[derive(Queryable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(CollectionModel, foreign_key = collection_id))]
#[diesel(belongs_to(crate::modules::anime::infrastructure::models::Anime, foreign_key = anime_id))]
#[diesel(table_name = collection_anime)]
#[diesel(primary_key(collection_id, anime_id))]
pub struct CollectionAnime {
    pub collection_id: Uuid,
    pub anime_id: Uuid,
    pub added_at: DateTime<Utc>,
    pub user_score: Option<f32>,
    pub notes: Option<String>,
}

// For inserting new collection-anime relationships
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = collection_anime)]
pub struct NewCollectionAnime {
    pub collection_id: Uuid,
    pub anime_id: Uuid,
    pub user_score: Option<f32>,
    pub notes: Option<String>,
}

// For updating existing collection-anime relationships
#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = collection_anime)]
pub struct CollectionAnimeChangeset {
    pub user_score: Option<f32>,
    pub notes: Option<String>,
}
