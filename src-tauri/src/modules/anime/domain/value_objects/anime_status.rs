use serde::{Deserialize, Serialize};
use specta::Type;
use std::fmt;
use std::str::FromStr;

#[derive(
    diesel_derive_enum::DbEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type,
)]
#[ExistingTypePath = "crate::schema::sql_types::AnimeStatus"]
pub enum AnimeStatus {
    Airing,
    Finished,
    NotYetAired,
    Cancelled,
    Unknown,
}

impl AnimeStatus {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            AnimeStatus::Airing => "Currently Airing",
            AnimeStatus::Finished => "Finished Airing",
            AnimeStatus::NotYetAired => "Not Yet Aired",
            AnimeStatus::Cancelled => "Cancelled",
            AnimeStatus::Unknown => "Unknown",
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, AnimeStatus::Unknown)
    }
}

impl fmt::Display for AnimeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<&str> for AnimeStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "currently airing" | "airing" => AnimeStatus::Airing,
            "finished airing" | "finished" => AnimeStatus::Finished,
            "not yet aired" | "not_yet_aired" => AnimeStatus::NotYetAired,
            "cancelled" => AnimeStatus::Cancelled,
            _ => AnimeStatus::Unknown,
        }
    }
}

impl From<String> for AnimeStatus {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl FromStr for AnimeStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}
