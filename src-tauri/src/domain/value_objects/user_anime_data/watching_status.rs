//! Watching status enum and display methods

use serde::{Deserialize, Serialize};
use specta::Type;

/// User's watching status for an anime
#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[ExistingTypePath = "crate::infrastructure::database::schema::sql_types::WatchingStatus"]
pub enum WatchingStatus {
    /// Planning to watch
    PlanToWatch,
    /// Currently watching
    Watching,
    /// Completed watching
    Completed,
    /// On hold/paused
    OnHold,
    /// Dropped
    Dropped,
    /// Re-watching
    Rewatching,
}

impl WatchingStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PlanToWatch => "Plan to Watch",
            Self::Watching => "Watching",
            Self::Completed => "Completed",
            Self::OnHold => "On Hold",
            Self::Dropped => "Dropped",
            Self::Rewatching => "Re-watching",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::PlanToWatch => "📋",
            Self::Watching => "▶️",
            Self::Completed => "✅",
            Self::OnHold => "⏸️",
            Self::Dropped => "❌",
            Self::Rewatching => "🔄",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            Self::PlanToWatch => "text-blue-500",
            Self::Watching => "text-green-500",
            Self::Completed => "text-purple-500",
            Self::OnHold => "text-yellow-500",
            Self::Dropped => "text-red-500",
            Self::Rewatching => "text-indigo-500",
        }
    }
}
