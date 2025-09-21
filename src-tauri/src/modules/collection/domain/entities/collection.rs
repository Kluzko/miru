use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub anime_ids: Vec<Uuid>,
    pub anime_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CollectionAnime {
    pub collection_id: Uuid,
    pub anime_id: Uuid,
    pub added_at: DateTime<Utc>,
    pub user_score: Option<f32>,
    pub notes: Option<String>,
}

impl Collection {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            anime_ids: Vec::new(),
            anime_count: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn add_anime(&mut self, anime_id: Uuid) -> bool {
        if !self.anime_ids.contains(&anime_id) {
            self.anime_ids.push(anime_id);
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn remove_anime(&mut self, anime_id: &Uuid) -> bool {
        let original_len = self.anime_ids.len();
        self.anime_ids.retain(|id| id != anime_id);

        if self.anime_ids.len() < original_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn contains_anime(&self, anime_id: &Uuid) -> bool {
        self.anime_ids.contains(anime_id)
    }

    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
        self.updated_at = Utc::now();
    }

    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }
}

impl CollectionAnime {
    pub fn update_score(&mut self, score: Option<f32>) {
        self.user_score = score;
    }

    pub fn update_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
    }
}
