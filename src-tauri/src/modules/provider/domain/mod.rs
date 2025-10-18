pub mod entities;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-exports for easy access
pub use entities::*;
pub use services::{AnimeSearchService, ProviderSelectionService};
pub use value_objects::{AnimeProvider, GetDetailsCriteria, ProviderMetadata, SearchCriteria};
