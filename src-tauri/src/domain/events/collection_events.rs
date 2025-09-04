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
