use serde::{Deserialize, Serialize};
use specta::Type;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AnimeStatus {
    Airing,
    Finished,
    NotYetAired,
    Unknown,
}

impl AnimeStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            AnimeStatus::Airing => "Currently Airing",
            AnimeStatus::Finished => "Finished Airing",
            AnimeStatus::NotYetAired => "Not Yet Aired",
            AnimeStatus::Unknown => "Unknown",
        }
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
