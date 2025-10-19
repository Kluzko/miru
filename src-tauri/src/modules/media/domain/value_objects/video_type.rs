use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, DbEnum, Type)]
#[ExistingTypePath = "crate::schema::sql_types::VideoType"]
#[serde(rename_all = "snake_case")]
pub enum VideoType {
    Trailer,
    Teaser,
    Clip,
    Opening,
    Ending,
    #[serde(rename = "pv")]
    PV,
    #[serde(rename = "cm")]
    CM,
    #[serde(rename = "behind_the_scenes")]
    BehindTheScenes,
    Featurette,
}

impl VideoType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VideoType::Trailer => "trailer",
            VideoType::Teaser => "teaser",
            VideoType::Clip => "clip",
            VideoType::Opening => "opening",
            VideoType::Ending => "ending",
            VideoType::PV => "pv",
            VideoType::CM => "cm",
            VideoType::BehindTheScenes => "behind_the_scenes",
            VideoType::Featurette => "featurette",
        }
    }

    /// Returns whether this video type is typically promotional content
    pub fn is_promotional(&self) -> bool {
        matches!(
            self,
            VideoType::Trailer
                | VideoType::Teaser
                | VideoType::PV
                | VideoType::CM
                | VideoType::Featurette
        )
    }

    /// Returns whether this video type is from the actual anime content
    pub fn is_content(&self) -> bool {
        matches!(
            self,
            VideoType::Opening | VideoType::Ending | VideoType::Clip
        )
    }
}

impl std::fmt::Display for VideoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
