use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimeTier {
    pub name: String,
    pub level: u8,
    pub color: String,
}

impl AnimeTier {
    pub fn new(score: f32) -> Self {
        if score >= 8.5 {
            Self {
                name: "GOAT".to_string(),
                level: 1,
                color: "#FFD700".to_string(),
            }
        } else if score >= 7.5 {
            Self {
                name: "Excellent".to_string(),
                level: 2,
                color: "#32CD32".to_string(),
            }
        } else if score >= 6.5 {
            Self {
                name: "Good".to_string(),
                level: 3,
                color: "#4169E1".to_string(),
            }
        } else if score >= 5.5 {
            Self {
                name: "Average".to_string(),
                level: 4,
                color: "#FFA500".to_string(),
            }
        } else if score >= 4.5 {
            Self {
                name: "Below Average".to_string(),
                level: 5,
                color: "#FF6347".to_string(),
            }
        } else {
            Self {
                name: "Poor".to_string(),
                level: 6,
                color: "#DC143C".to_string(),
            }
        }
    }

    pub fn is_top_tier(&self) -> bool {
        self.level <= 2
    }

    pub fn is_recommended(&self) -> bool {
        self.level <= 3
    }
}

impl Default for AnimeTier {
    fn default() -> Self {
        Self {
            name: "Unrated".to_string(),
            level: 0,
            color: "#808080".to_string(),
        }
    }
}

impl fmt::Display for AnimeTier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
