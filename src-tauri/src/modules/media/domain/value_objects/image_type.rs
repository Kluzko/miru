use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, DbEnum, Type)]
#[ExistingTypePath = "crate::schema::sql_types::ImageType"]
#[serde(rename_all = "lowercase")]
pub enum ImageType {
    #[serde(rename = "poster")]
    Poster,
    #[serde(rename = "backdrop")]
    Backdrop,
    #[serde(rename = "logo")]
    Logo,
    #[serde(rename = "still")]
    Still,
    #[serde(rename = "banner")]
    Banner,
    #[serde(rename = "cover")]
    Cover,
}

impl ImageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageType::Poster => "poster",
            ImageType::Backdrop => "backdrop",
            ImageType::Logo => "logo",
            ImageType::Still => "still",
            ImageType::Banner => "banner",
            ImageType::Cover => "cover",
        }
    }

    /// Returns the typical aspect ratio for this image type
    pub fn typical_aspect_ratio(&self) -> Option<f32> {
        match self {
            ImageType::Poster => Some(2.0 / 3.0),    // 2:3 (portrait)
            ImageType::Backdrop => Some(16.0 / 9.0), // 16:9 (landscape)
            ImageType::Banner => Some(21.0 / 9.0),   // 21:9 (ultra-wide)
            ImageType::Logo => None,                 // Variable
            ImageType::Still => Some(16.0 / 9.0),    // 16:9 (landscape)
            ImageType::Cover => Some(2.0 / 3.0),     // 2:3 (portrait)
        }
    }
}

impl std::fmt::Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
