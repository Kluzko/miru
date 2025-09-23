pub mod entities;
pub mod repositories;
pub mod services;
pub mod traits;
pub mod value_objects;

// Re-exports for easy access
pub use entities::anime_detailed::AnimeDetailed;
pub use repositories::anime_repository::AnimeRepository;
