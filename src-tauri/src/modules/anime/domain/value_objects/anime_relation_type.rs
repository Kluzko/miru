use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(
    diesel_derive_enum::DbEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type,
)]
#[ExistingTypePath = "crate::schema::sql_types::AnimeRelationType"]
pub enum AnimeRelationType {
    // AniList relative relation types (legacy support)
    #[db_rename = "sequel"]
    Sequel,
    #[db_rename = "prequel"]
    Prequel,
    #[db_rename = "side_story"]
    SideStory,
    #[db_rename = "spin_off"]
    SpinOff,
    #[db_rename = "alternative"]
    Alternative,
    #[db_rename = "summary"]
    Summary,
    #[db_rename = "special"]
    Special,
    #[db_rename = "movie"]
    Movie,
    #[db_rename = "parent_story"]
    ParentStory,
    #[db_rename = "full_story"]
    FullStory,
    #[db_rename = "same_setting"]
    SameSetting,
    #[db_rename = "shared_character"]
    SharedCharacter,

    // Semantic franchise categories (absolute categorization)
    #[db_rename = "main_story"]
    MainStory,
    #[db_rename = "movies"]
    Movies,
    #[db_rename = "ova_special"]
    OvaSpecial,

    #[db_rename = "other"]
    Other,
}

impl AnimeRelationType {
    /// Get the database representation (matches the enum values in PostgreSQL)
    pub fn db_value(&self) -> &'static str {
        match self {
            AnimeRelationType::Sequel => "sequel",
            AnimeRelationType::Prequel => "prequel",
            AnimeRelationType::SideStory => "side_story",
            AnimeRelationType::SpinOff => "spin_off",
            AnimeRelationType::Alternative => "alternative",
            AnimeRelationType::Summary => "summary",
            AnimeRelationType::Special => "special",
            AnimeRelationType::Movie => "movie",
            AnimeRelationType::ParentStory => "parent_story",
            AnimeRelationType::FullStory => "full_story",
            AnimeRelationType::SameSetting => "same_setting",
            AnimeRelationType::SharedCharacter => "shared_character",
            AnimeRelationType::MainStory => "main_story",
            AnimeRelationType::Movies => "movies",
            AnimeRelationType::OvaSpecial => "ova_special",
            AnimeRelationType::Other => "other",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AnimeRelationType::Sequel => "Sequel",
            AnimeRelationType::Prequel => "Prequel",
            AnimeRelationType::SideStory => "Side Story",
            AnimeRelationType::SpinOff => "Spin-off",
            AnimeRelationType::Alternative => "Alternative Version",
            AnimeRelationType::Summary => "Summary",
            AnimeRelationType::Special => "Special",
            AnimeRelationType::Movie => "Movie",
            AnimeRelationType::ParentStory => "Parent Story",
            AnimeRelationType::FullStory => "Full Story",
            AnimeRelationType::SameSetting => "Same Setting",
            AnimeRelationType::SharedCharacter => "Shared Character",
            AnimeRelationType::MainStory => "Main Story",
            AnimeRelationType::Movies => "Movies",
            AnimeRelationType::OvaSpecial => "OVA/Special",
            AnimeRelationType::Other => "Other",
        }
    }

    /// Get the inverse relationship type
    pub fn inverse(&self) -> Self {
        match self {
            AnimeRelationType::Sequel => AnimeRelationType::Prequel,
            AnimeRelationType::Prequel => AnimeRelationType::Sequel,
            AnimeRelationType::ParentStory => AnimeRelationType::SideStory,
            AnimeRelationType::SideStory => AnimeRelationType::ParentStory,
            AnimeRelationType::FullStory => AnimeRelationType::Summary,
            AnimeRelationType::Summary => AnimeRelationType::FullStory,
            // Symmetric relationships
            AnimeRelationType::SpinOff => AnimeRelationType::SpinOff,
            AnimeRelationType::Alternative => AnimeRelationType::Alternative,
            AnimeRelationType::Special => AnimeRelationType::Special,
            AnimeRelationType::Movie => AnimeRelationType::Movie,
            AnimeRelationType::SameSetting => AnimeRelationType::SameSetting,
            AnimeRelationType::SharedCharacter => AnimeRelationType::SharedCharacter,
            // Semantic categories are symmetric (franchise-wide)
            AnimeRelationType::MainStory => AnimeRelationType::MainStory,
            AnimeRelationType::Movies => AnimeRelationType::Movies,
            AnimeRelationType::OvaSpecial => AnimeRelationType::OvaSpecial,
            AnimeRelationType::Other => AnimeRelationType::Other,
        }
    }

    /// Check if this relationship type indicates chronological order
    pub fn is_chronological(&self) -> bool {
        matches!(self, AnimeRelationType::Sequel | AnimeRelationType::Prequel)
    }
}

impl std::fmt::Display for AnimeRelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
