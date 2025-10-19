pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types
pub use application::{MediaService, MediaStats};
pub use domain::{
    AnimeImage, AnimeImageRepository, AnimeProvider, AnimeVideo, AnimeVideoRepository, ImageType,
    NewAnimeImage, NewAnimeVideo, VideoType,
};
pub use infrastructure::{AnimeImageRepositoryImpl, AnimeVideoRepositoryImpl};
