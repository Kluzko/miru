use async_trait::async_trait;

use crate::modules::provider::infrastructure::adapters::anilist::models::{
    CategorizedFranchise, FranchiseRelation,
};
use crate::shared::errors::AppResult;

/// Repository interface for fetching anime relationships and franchise information
///
/// This abstracts the relationship discovery functionality, allowing the application
/// layer to fetch relationship data without depending on specific provider adapters.
///
/// Currently, only AniList provides comprehensive relationship data through its
/// GraphQL API, but this abstraction allows for future providers to be added.
#[async_trait]
pub trait RelationshipProviderRepository: Send + Sync {
    /// Get basic anime relations (ID and relation type)
    ///
    /// Returns a list of related anime IDs with their relationship types
    /// (e.g., "SEQUEL", "PREQUEL", "SIDE_STORY")
    async fn get_anime_relations(&self, anime_id: u32) -> AppResult<Vec<(u32, String)>>;

    /// Discover complete franchise details with full relationship information
    ///
    /// Performs deep franchise discovery, returning detailed information about
    /// all related anime in the franchise
    async fn discover_franchise_details(&self, anime_id: u32) -> AppResult<Vec<FranchiseRelation>>;

    /// Discover franchise with categorization by relationship type
    ///
    /// Returns franchise information organized by relationship categories
    /// (main story, side stories, etc.)
    async fn discover_categorized_franchise(
        &self,
        anime_id: u32,
    ) -> AppResult<CategorizedFranchise>;

    /// Check if this repository supports relationship discovery
    ///
    /// Returns true if the underlying provider has relationship data available
    fn supports_relationships(&self) -> bool;
}
