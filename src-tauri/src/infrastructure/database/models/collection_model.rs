use crate::infrastructure::database::schema::{collection_anime, collections};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
