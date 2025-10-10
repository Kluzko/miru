// Clean architecture modules
pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

// Primary exports - Clean Architecture
pub use application::service::ProviderService;
pub use commands::*;

pub use domain::value_objects::*;
pub use infrastructure::adapters::{AniListAdapter, JikanAdapter, ProviderAdapter};
