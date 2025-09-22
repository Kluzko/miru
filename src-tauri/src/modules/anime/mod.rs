pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

// Re-exports for easy external access
pub use application::service::AnimeService;
pub use domain::{AnimeDetailed, AnimeRepository};

// Re-export common value objects for shorter imports
pub use domain::value_objects::{AnimeStatus, AnimeTier, AnimeType, UnifiedAgeRestriction};

// Re-export infrastructure components
pub use infrastructure::mappers::age_restriction_mapper::AgeRestrictionMapper;
