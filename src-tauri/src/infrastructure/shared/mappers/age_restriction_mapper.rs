use crate::domain::value_objects::{AnimeProvider, UnifiedAgeRestriction};

/// Maps provider-specific age restriction values to unified age restrictions
/// Used only during data ingestion from external providers
pub struct AgeRestrictionMapper;

impl AgeRestrictionMapper {
    /// Map provider-specific age restriction to unified format
    /// This is called during data ingestion to convert provider values immediately
    pub fn map_to_unified(
        provider: &AnimeProvider,
        provider_value: &str,
    ) -> Option<UnifiedAgeRestriction> {
        match provider {
            AnimeProvider::Jikan => Self::map_jikan_rating(provider_value),
            AnimeProvider::AniList => Self::map_anilist_rating(provider_value),
            AnimeProvider::Kitsu => Self::map_kitsu_rating(provider_value),
            AnimeProvider::TMDB => Self::map_tmdb_rating(provider_value),
            AnimeProvider::AniDB => Self::map_anidb_rating(provider_value),
        }
    }

    /// Map Jikan (MyAnimeList) rating values
    fn map_jikan_rating(rating: &str) -> Option<UnifiedAgeRestriction> {
        match rating.to_lowercase().as_str() {
            "g" | "g - all ages" => Some(UnifiedAgeRestriction::GeneralAudiences),
            "pg" | "pg - children" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "pg-13" | "pg-13 - teens 13 or older" => {
                Some(UnifiedAgeRestriction::ParentalGuidance13)
            }
            "r" | "r - 17+" | "r+ - mild nudity" => Some(UnifiedAgeRestriction::Mature),
            "rx" | "rx - hentai" => Some(UnifiedAgeRestriction::Explicit),
            _ => None,
        }
    }

    /// Map AniList rating values
    fn map_anilist_rating(rating: &str) -> Option<UnifiedAgeRestriction> {
        match rating.to_lowercase().as_str() {
            "g" | "all ages" => Some(UnifiedAgeRestriction::GeneralAudiences),
            "pg" | "parental guidance" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "pg13" | "pg-13" | "13+" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "r" | "17+" => Some(UnifiedAgeRestriction::Mature),
            "18+" | "adult" => Some(UnifiedAgeRestriction::Explicit),
            _ => None,
        }
    }

    /// Map Kitsu rating values
    fn map_kitsu_rating(rating: &str) -> Option<UnifiedAgeRestriction> {
        match rating.to_lowercase().as_str() {
            "g" | "general" => Some(UnifiedAgeRestriction::GeneralAudiences),
            "pg" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "pg13" | "pg-13" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "r" => Some(UnifiedAgeRestriction::Mature),
            "r18" | "r18+" => Some(UnifiedAgeRestriction::Explicit),
            _ => None,
        }
    }

    /// Map TMDB rating values
    fn map_tmdb_rating(rating: &str) -> Option<UnifiedAgeRestriction> {
        match rating.to_lowercase().as_str() {
            "g" => Some(UnifiedAgeRestriction::GeneralAudiences),
            "pg" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "pg-13" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "r" => Some(UnifiedAgeRestriction::Mature),
            "nc-17" => Some(UnifiedAgeRestriction::Explicit),
            _ => None,
        }
    }

    /// Map AniDB rating values
    fn map_anidb_rating(rating: &str) -> Option<UnifiedAgeRestriction> {
        match rating.to_lowercase().as_str() {
            "all" | "all ages" => Some(UnifiedAgeRestriction::GeneralAudiences),
            "pg" | "parental guidance" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "13+" | "teen" => Some(UnifiedAgeRestriction::ParentalGuidance13),
            "17+" | "mature" => Some(UnifiedAgeRestriction::Mature),
            "18+" | "adult" | "restricted" => Some(UnifiedAgeRestriction::Explicit),
            _ => None,
        }
    }
}
