use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimeType {
    TV,
    Movie,
    OVA,
    Special,
    ONA,
    Music,
    Unknown,
}

impl AnimeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnimeType::TV => "TV",
            AnimeType::Movie => "Movie",
            AnimeType::OVA => "OVA",
            AnimeType::Special => "Special",
            AnimeType::ONA => "ONA",
            AnimeType::Music => "Music",
            AnimeType::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for AnimeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for AnimeType {
    fn from(s: &str) -> Self {
        match s {
            "TV" => AnimeType::TV,
            "Movie" => AnimeType::Movie,
            "OVA" => AnimeType::OVA,
            "Special" => AnimeType::Special,
            "ONA" => AnimeType::ONA,
            "Music" => AnimeType::Music,
            _ => AnimeType::Unknown,
        }
    }
}

impl From<String> for AnimeType {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl FromStr for AnimeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}
