use serde::{Deserialize, Serialize};
use specta::Type;
use std::fmt;

#[derive(
    diesel_derive_enum::DbEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type,
)]
#[ExistingTypePath = "crate::schema::sql_types::AnimeTier"]
pub enum AnimeTier {
    S, // Best tier
    A,
    B,
    C,
    D, // Worst tier
}

impl AnimeTier {
    pub fn from_score(score: f32) -> Self {
        if score >= 9.0 {
            AnimeTier::S
        } else if score >= 8.0 {
            AnimeTier::A
        } else if score >= 7.0 {
            AnimeTier::B
        } else if score >= 6.0 {
            AnimeTier::C
        } else {
            AnimeTier::D
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AnimeTier::S => "S-Tier (GOAT)",
            AnimeTier::A => "A-Tier (Excellent)",
            AnimeTier::B => "B-Tier (Good)",
            AnimeTier::C => "C-Tier (Average)",
            AnimeTier::D => "D-Tier (Poor)",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            AnimeTier::S => "#FFD700", // Gold
            AnimeTier::A => "#32CD32", // Lime Green
            AnimeTier::B => "#4169E1", // Royal Blue
            AnimeTier::C => "#FFA500", // Orange
            AnimeTier::D => "#DC143C", // Crimson
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            AnimeTier::S => 1,
            AnimeTier::A => 2,
            AnimeTier::B => 3,
            AnimeTier::C => 4,
            AnimeTier::D => 5,
        }
    }
}

impl Default for AnimeTier {
    fn default() -> Self {
        AnimeTier::C
    }
}

impl fmt::Display for AnimeTier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
