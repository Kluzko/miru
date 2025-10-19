pub mod anilist;
pub mod cache_adapter;
pub mod jikan;
pub mod provider_repository_adapter;
pub mod tmdb;

// Use specific imports to avoid conflicts
pub use anilist::AniListAdapter;
pub use cache_adapter::*;
pub use jikan::JikanAdapter;
pub use provider_repository_adapter::*;
pub use tmdb::TmdbAdapter;
