use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum CollectionEvent {
    CollectionCreated {
        collection_id: Uuid,
        name: String,
        created_at: DateTime<Utc>,
    },
    CollectionUpdated {
        collection_id: Uuid,
        updated_fields: Vec<String>,
        updated_at: DateTime<Utc>,
    },
    CollectionDeleted {
        collection_id: Uuid,
        deleted_at: DateTime<Utc>,
    },
    CollectionRenamed {
        collection_id: Uuid,
        old_name: String,
        new_name: String,
        renamed_at: DateTime<Utc>,
    },
    CollectionCleared {
        collection_id: Uuid,
        anime_count: usize,
        cleared_at: DateTime<Utc>,
    },
}

impl CollectionEvent {
    pub fn collection_created(collection_id: Uuid, name: String) -> Self {
        Self::CollectionCreated {
            collection_id,
            name,
            created_at: Utc::now(),
        }
    }

    pub fn collection_updated(collection_id: Uuid, updated_fields: Vec<String>) -> Self {
        Self::CollectionUpdated {
            collection_id,
            updated_fields,
            updated_at: Utc::now(),
        }
    }

    pub fn collection_deleted(collection_id: Uuid) -> Self {
        Self::CollectionDeleted {
            collection_id,
            deleted_at: Utc::now(),
        }
    }

    pub fn collection_renamed(collection_id: Uuid, old_name: String, new_name: String) -> Self {
        Self::CollectionRenamed {
            collection_id,
            old_name,
            new_name,
            renamed_at: Utc::now(),
        }
    }

    pub fn collection_cleared(collection_id: Uuid, anime_count: usize) -> Self {
        Self::CollectionCleared {
            collection_id,
            anime_count,
            cleared_at: Utc::now(),
        }
    }

    pub fn event_timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::CollectionCreated { created_at, .. } => *created_at,
            Self::CollectionUpdated { updated_at, .. } => *updated_at,
            Self::CollectionDeleted { deleted_at, .. } => *deleted_at,
            Self::CollectionRenamed { renamed_at, .. } => *renamed_at,
            Self::CollectionCleared { cleared_at, .. } => *cleared_at,
        }
    }
}
