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

impl AnimeEvent {
    pub fn anime_created(anime_id: Uuid, title: String) -> Self {
        Self::AnimeCreated {
            anime_id,
            title,
            created_at: Utc::now(),
        }
    }

    pub fn anime_updated(anime_id: Uuid, updated_fields: Vec<String>) -> Self {
        Self::AnimeUpdated {
            anime_id,
            updated_fields,
            updated_at: Utc::now(),
        }
    }

    pub fn anime_deleted(anime_id: Uuid) -> Self {
        Self::AnimeDeleted {
            anime_id,
            deleted_at: Utc::now(),
        }
    }

    pub fn score_updated(anime_id: Uuid, old_score: f32, new_score: f32) -> Self {
        Self::AnimeScoreUpdated {
            anime_id,
            old_score,
            new_score,
            updated_at: Utc::now(),
        }
    }

    pub fn added_to_collection(anime_id: Uuid, collection_id: Uuid) -> Self {
        Self::AnimeAddedToCollection {
            anime_id,
            collection_id,
            added_at: Utc::now(),
        }
    }

    pub fn removed_from_collection(anime_id: Uuid, collection_id: Uuid) -> Self {
        Self::AnimeRemovedFromCollection {
            anime_id,
            collection_id,
            removed_at: Utc::now(),
        }
    }

    pub fn bulk_imported(anime_ids: Vec<Uuid>) -> Self {
        let count = anime_ids.len();
        Self::AnimeBulkImported {
            anime_ids,
            count,
            imported_at: Utc::now(),
        }
    }

    pub fn event_timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::AnimeCreated { created_at, .. } => *created_at,
            Self::AnimeUpdated { updated_at, .. } => *updated_at,
            Self::AnimeDeleted { deleted_at, .. } => *deleted_at,
            Self::AnimeScoreUpdated { updated_at, .. } => *updated_at,
            Self::AnimeAddedToCollection { added_at, .. } => *added_at,
            Self::AnimeRemovedFromCollection { removed_at, .. } => *removed_at,
            Self::AnimeBulkImported { imported_at, .. } => *imported_at,
        }
    }
}
