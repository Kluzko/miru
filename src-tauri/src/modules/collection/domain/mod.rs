pub mod entities;
pub mod repositories;

// Re-exports for easy access
pub use entities::collection::{Collection, CollectionAnime};
pub use entities::user_rating::UserRating;
pub use repositories::collection_repository::CollectionRepository;
