use serde::{Deserialize, Serialize};
use specta::datatype::PrimitiveType;
use specta::{DataType, Generics, Type, TypeCollection};

/// Unified age restriction system that maps all provider age restrictions to a common standard
#[derive(diesel_derive_enum::DbEnum, Debug, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::UnifiedAgeRestriction"]
pub enum UnifiedAgeRestriction {
    /// G - All Ages (0+)
    GeneralAudiences,
    /// PG13 - Parental Guidance for 13+
    ParentalGuidance13,
    /// PG17 - Parental Guidance for 17+
    ParentalGuidance17,
    /// Mature - Mature content (17+)
    Mature,
    /// Explicit - Explicit sexual content (18+)
    Explicit,
}

impl UnifiedAgeRestriction {
    /// Get age recommendation
    pub fn min_age(&self) -> u8 {
        match self {
            Self::GeneralAudiences => 0,
            Self::ParentalGuidance13 => 13,
            Self::ParentalGuidance17 => 17,
            Self::Mature => 17,
            Self::Explicit => 18,
        }
    }

    /// Display name for UI
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::GeneralAudiences => "All Ages",
            Self::ParentalGuidance13 => "Teens (13+)",
            Self::ParentalGuidance17 => "Teens (17+)",
            Self::Mature => "Mature (17+)",
            Self::Explicit => "Explicit (18+)",
        }
    }

    /// Description for UI
    pub fn description(&self) -> &'static str {
        match self {
            Self::GeneralAudiences => "Suitable for everyone",
            Self::ParentalGuidance13 => "May contain mild violence, language, or suggestive themes",
            Self::ParentalGuidance17 => "May contain stronger themes suitable for older teens",
            Self::Mature => "Stronger violence, harsh language, mature themes",
            Self::Explicit => "Explicit sexual content",
        }
    }
}

impl Default for UnifiedAgeRestriction {
    fn default() -> Self {
        Self::GeneralAudiences
    }
}

impl std::fmt::Display for UnifiedAgeRestriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<UnifiedAgeRestriction> for String {
    fn from(restriction: UnifiedAgeRestriction) -> Self {
        restriction.display_name().to_string()
    }
}

impl std::str::FromStr for UnifiedAgeRestriction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "All Ages" => Ok(Self::GeneralAudiences),
            "Teens (13+)" => Ok(Self::ParentalGuidance13),
            "Teens (17+)" => Ok(Self::ParentalGuidance17),
            "Mature (17+)" => Ok(Self::Mature),
            "Explicit (18+)" => Ok(Self::Explicit),
            // Fallback to variant names for backward compatibility
            "GeneralAudiences" => Ok(Self::GeneralAudiences),
            "ParentalGuidance13" => Ok(Self::ParentalGuidance13),
            "ParentalGuidance17" => Ok(Self::ParentalGuidance17),
            "Mature" => Ok(Self::Mature),
            "Explicit" => Ok(Self::Explicit),
            _ => Err(format!("Unknown age restriction: {}", s)),
        }
    }
}

impl TryFrom<String> for UnifiedAgeRestriction {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// Age restriction display wrapper for frontend serialization
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AgeRestrictionDisplay(pub String);

impl From<UnifiedAgeRestriction> for AgeRestrictionDisplay {
    fn from(restriction: UnifiedAgeRestriction) -> Self {
        Self(restriction.display_name().to_string())
    }
}

impl Serialize for UnifiedAgeRestriction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.display_name())
    }
}

impl<'de> Deserialize<'de> for UnifiedAgeRestriction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl Type for UnifiedAgeRestriction {
    fn inline(_type_map: &mut TypeCollection, _generics: Generics) -> DataType {
        // Generate a string type with the possible display names
        DataType::Primitive(PrimitiveType::String)
    }
}

/// Age restriction information for frontend display
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AgeRestrictionInfo {
    /// The enum variant name
    pub variant: UnifiedAgeRestriction,
    /// User-friendly display name
    pub display_name: String,
    /// Description for UI
    pub description: String,
    /// Minimum age
    pub min_age: u8,
}

impl From<UnifiedAgeRestriction> for AgeRestrictionInfo {
    fn from(restriction: UnifiedAgeRestriction) -> Self {
        Self {
            display_name: restriction.display_name().to_string(),
            description: restriction.description().to_string(),
            min_age: restriction.min_age(),
            variant: restriction,
        }
    }
}

impl UnifiedAgeRestriction {
    /// Get all age restrictions with display information
    pub fn all_with_info() -> Vec<AgeRestrictionInfo> {
        vec![
            Self::GeneralAudiences.into(),
            Self::ParentalGuidance13.into(),
            Self::ParentalGuidance17.into(),
            Self::Mature.into(),
            Self::Explicit.into(),
        ]
    }
}
