use serde::{Deserialize, Serialize};
use specta::Type;

/// Anime season enum for better type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub enum Season {
    Winter,
    Spring,
    Summer,
    Fall,
}

impl Season {
    /// Get season from string (case insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "winter" => Some(Self::Winter),
            "spring" => Some(Self::Spring),
            "summer" => Some(Self::Summer),
            "fall" | "autumn" => Some(Self::Fall),
            _ => None,
        }
    }

    /// Get season display name
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Winter => "Winter",
            Self::Spring => "Spring",
            Self::Summer => "Summer",
            Self::Fall => "Fall",
        }
    }

    /// Get season from month (1-12)
    pub fn from_month(month: u32) -> Option<Self> {
        match month {
            12 | 1 | 2 => Some(Self::Winter),
            3 | 4 | 5 => Some(Self::Spring),
            6 | 7 | 8 => Some(Self::Summer),
            9 | 10 | 11 => Some(Self::Fall),
            _ => None,
        }
    }

    /// Get all seasons in chronological order
    pub fn all() -> [Self; 4] {
        [Self::Winter, Self::Spring, Self::Summer, Self::Fall]
    }
}
