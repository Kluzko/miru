//! Anime-specific value objects

pub mod anime_status;
pub mod anime_tier;
pub mod anime_title;
pub mod anime_type;
pub mod quality_metrics;
pub mod unified_age_restriction;

pub use anime_status::AnimeStatus;
pub use anime_tier::AnimeTier;
pub use anime_title::AnimeTitle;
pub use anime_type::AnimeType;
pub use quality_metrics::QualityMetrics;
pub use unified_age_restriction::UnifiedAgeRestriction;
