//! User anime data management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

// Sub-modules
mod management;
mod stats;
mod watching_status;

// Re-export types
pub use stats::UserAnimeStats;
pub use watching_status::WatchingStatus;

/// User's personal data for an anime
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UserAnimeData {
    pub id: Uuid,
    pub anime_id: Uuid,
    pub user_id: String, // For now, can be more complex user system later

    /// Watching status
    pub status: WatchingStatus,

    /// User's personal rating (0-10)
    pub personal_rating: Option<f32>,

    /// Episodes watched
    pub episodes_watched: i32,

    /// Times rewatched
    pub rewatched_count: i32,

    /// Personal notes
    pub notes: Option<String>,

    /// Personal tags
    pub tags: Vec<String>,

    /// Favorite status
    pub is_favorite: bool,

    /// Date started watching
    pub start_date: Option<DateTime<Utc>>,

    /// Date completed watching
    pub finish_date: Option<DateTime<Utc>>,

    /// Last updated
    pub updated_at: DateTime<Utc>,

    /// Created date
    pub created_at: DateTime<Utc>,
}

impl UserAnimeData {
    pub fn new(anime_id: Uuid, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            anime_id,
            user_id,
            status: WatchingStatus::PlanToWatch,
            personal_rating: None,
            episodes_watched: 0,
            rewatched_count: 0,
            notes: None,
            tags: Vec::new(),
            is_favorite: false,
            start_date: None,
            finish_date: None,
            updated_at: now,
            created_at: now,
        }
    }
}
