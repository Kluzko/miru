// Clean architecture modules
pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

// Primary exports - Clean Architecture
// ONLY expose application layer - adapters are PRIVATE
pub use application::service::ProviderService;
pub use commands::*;
pub use domain::value_objects::*;

// ❌ REMOVED: Adapter exposure violates Clean Architecture
// pub use infrastructure::adapters::{AniListAdapter, JikanAdapter, ProviderAdapter};
