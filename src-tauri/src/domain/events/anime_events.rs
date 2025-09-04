use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum AnimeEvent {
    AnimeCreated {
        anime_id: Uuid,
        title: String,
        created_at: DateTime<Utc>,
    },
    AnimeUpdated {
        anime_id: Uuid,
        updated_fields: Vec<String>,
        updated_at: DateTime<Utc>,
    },
    AnimeDeleted {
        anime_id: Uuid,
        deleted_at: DateTime<Utc>,
    },
    AnimeScoreUpdated {
        anime_id: Uuid,
        old_score: f32,
        new_score: f32,
        updated_at: DateTime<Utc>,
    },
    AnimeAddedToCollection {
        anime_id: Uuid,
        collection_id: Uuid,
        added_at: DateTime<Utc>,
    },
    AnimeRemovedFromCollection {
        anime_id: Uuid,
        collection_id: Uuid,
        removed_at: DateTime<Utc>,
    },
    AnimeBulkImported {
        anime_ids: Vec<Uuid>,
        count: usize,
        imported_at: DateTime<Utc>,
    },
}
