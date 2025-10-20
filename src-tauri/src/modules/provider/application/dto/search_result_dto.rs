use serde::{Deserialize, Serialize};
use specta::Type;

use crate::modules::{
    anime::domain::entities::anime_detailed::AnimeDetailed, provider::domain::entities::AnimeData,
};
use crate::shared::domain::value_objects::AnimeProvider;

/// Rich DTO that preserves quality metadata for frontend
///
/// This DTO solves a critical problem: previously, AnimeData's rich quality metadata
/// (quality score, completeness, relevance, provenance) was thrown away when converting
/// to AnimeDetailed for the frontend. This meant the frontend couldn't show users:
/// - How confident we are in the data
/// - Which providers contributed data
/// - What fields are missing
/// - How relevant the result is to their search
///
/// # Design Pattern: DTO (Data Transfer Object)
/// Preserves all information across layer boundaries while keeping domain model clean.
///
/// # Frontend Benefits:
/// - Can show quality indicators (e.g., "95% complete data")
/// - Can display provider badges (e.g., "Data from AniList + TMDB")
/// - Can sort by relevance score
/// - Can show confidence levels
/// - Can indicate missing information
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultDTO {
    /// The anime entity with all details
    pub anime: AnimeDetailed,

    /// Quality metrics
    ///
    /// Quality score (0.0-1.0) indicates overall data quality based on:
    /// - Field completeness
    /// - Data consistency across providers
    /// - Metadata richness
    pub quality_score: f32,

    /// Completeness score (0.0-1.0)
    ///
    /// Indicates what percentage of expected fields are populated.
    /// E.g., 0.9 means 90% of fields have data.
    pub completeness: f32,

    /// Relevance score (0.0-100.0)
    ///
    /// Indicates how well this result matches the search query.
    /// Based on fuzzy title matching (Jaro-Winkler + Levenshtein).
    /// Higher scores = better match.
    pub relevance_score: f32,

    /// Provenance tracking
    ///
    /// Which providers contributed data to this result.
    /// E.g., ["AniList", "TMDB"] means data was merged from both.
    pub sources: Vec<AnimeProvider>,

    /// Confidence level (0.0-1.0)
    ///
    /// How confident we are that this is the correct anime.
    /// Based on cross-provider agreement and data consistency.
    pub confidence: f32,

    /// Transparency
    ///
    /// List of field names that are missing or incomplete.
    /// E.g., ["synopsis", "episodes"] means these fields lack data.
    pub missing_fields: Vec<String>,

    /// Performance tracking
    ///
    /// How long it took to fetch this data (milliseconds).
    /// Useful for debugging and optimization.
    pub fetch_time_ms: u64,
}

impl From<AnimeData> for SearchResultDTO {
    /// Convert AnimeData to SearchResultDTO, preserving all metadata
    fn from(data: AnimeData) -> Self {
        Self {
            anime: data.anime,
            quality_score: data.quality.score,
            completeness: data.quality.completeness,
            relevance_score: data.quality.relevance_score,
            sources: data.source.providers_used,
            confidence: data.source.confidence,
            missing_fields: data.quality.missing_fields,
            fetch_time_ms: data.source.fetch_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dto_preserves_quality_metadata() {
        // Test would verify that all quality metadata is preserved
        // This is a placeholder for the test structure
    }

    #[test]
    fn test_dto_serialization() {
        // Test would verify that DTO serializes correctly for frontend
        // This is a placeholder for the test structure
    }
}
