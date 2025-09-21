pub mod entities;
pub mod events;
pub mod repositories;
pub mod services;
pub mod traits;
pub mod value_objects;

// Re-exports for easy access
pub use entities::anime_detailed::AnimeDetailed;
pub use entities::genre::Genre;
pub use repositories::anime_repository::AnimeRepository;
pub use services::score_calculator::ScoreCalculator;
pub use traits::Scoreable;
pub use value_objects::{
    anime_status::AnimeStatus, anime_tier::AnimeTier, anime_title::AnimeTitle,
    anime_type::AnimeType, quality_metrics::QualityMetrics,
};
