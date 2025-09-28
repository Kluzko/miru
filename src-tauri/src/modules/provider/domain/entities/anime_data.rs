use crate::modules::anime::domain::entities::anime_detailed::AnimeDetailed;
use crate::modules::provider::AnimeProvider;
use serde::{Deserialize, Serialize};
use specta::Type;

/// Wrapper for anime data with quality and source metadata
/// This follows the existing architecture where AnimeDetailed is the main entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AnimeData {
    /// The main anime entity
    pub anime: AnimeDetailed,

    /// Quality assessment of the data
    pub quality: DataQuality,

    /// Information about data sources
    pub source: DataSource,
}

/// Data quality metrics for anime data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DataQuality {
    /// Overall quality score (0.0 to 1.0)
    pub score: f32,

    /// Completeness percentage (0.0 to 1.0)
    pub completeness: f32,

    /// Data consistency score (0.0 to 1.0)
    pub consistency: f32,

    /// Search relevance score (0.0 to 100.0)
    pub relevance_score: f32,

    /// List of missing critical fields
    pub missing_fields: Vec<String>,
}

/// Information about data sources and fetching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DataSource {
    /// Primary provider used for this data
    pub primary_provider: AnimeProvider,

    /// All providers that contributed data
    pub providers_used: Vec<AnimeProvider>,

    /// Confidence in the data accuracy (0.0 to 1.0)
    pub confidence: f32,

    /// Time taken to fetch the data (in milliseconds)
    pub fetch_time_ms: u64,
}

impl DataQuality {
    /// Calculate quality metrics for anime data
    pub fn calculate(anime: &AnimeDetailed) -> Self {
        let mut missing_fields = Vec::new();
        let mut field_count = 0;
        let mut filled_count = 0;

        // Check critical fields
        if anime.title.main.is_empty() {
            missing_fields.push("title".to_string());
        } else {
            filled_count += 1;
        }
        field_count += 1;

        if anime.synopsis.is_none() {
            missing_fields.push("synopsis".to_string());
        } else {
            filled_count += 1;
        }
        field_count += 1;

        if anime.episodes.is_none() {
            missing_fields.push("episodes".to_string());
        } else {
            filled_count += 1;
        }
        field_count += 1;

        if anime.score.is_none() {
            missing_fields.push("score".to_string());
        } else {
            filled_count += 1;
        }
        field_count += 1;

        if anime.genres.is_empty() {
            missing_fields.push("genres".to_string());
        } else {
            filled_count += 1;
        }
        field_count += 1;

        let completeness = filled_count as f32 / field_count as f32;
        let consistency = if missing_fields.is_empty() { 1.0 } else { 0.8 };
        let score = (completeness * 0.6 + consistency * 0.4).min(1.0).max(0.0);

        Self {
            score,
            completeness,
            consistency,
            relevance_score: 0.0, // Default, will be set during search ranking
            missing_fields,
        }
    }

    /// Merge multiple quality assessments
    pub fn merge(qualities: &[DataQuality]) -> Self {
        if qualities.is_empty() {
            return Self::default();
        }

        if qualities.len() == 1 {
            return qualities[0].clone();
        }

        let avg_score = qualities.iter().map(|q| q.score).sum::<f32>() / qualities.len() as f32;
        let avg_completeness =
            qualities.iter().map(|q| q.completeness).sum::<f32>() / qualities.len() as f32;
        let avg_consistency =
            qualities.iter().map(|q| q.consistency).sum::<f32>() / qualities.len() as f32;

        let mut all_missing_fields = std::collections::HashSet::new();
        for quality in qualities {
            for field in &quality.missing_fields {
                all_missing_fields.insert(field.clone());
            }
        }

        Self {
            score: avg_score,
            completeness: avg_completeness,
            consistency: avg_consistency,
            relevance_score: 0.0, // Will be recalculated if needed
            missing_fields: all_missing_fields.into_iter().collect(),
        }
    }
}

impl Default for DataQuality {
    fn default() -> Self {
        Self {
            score: 0.8,
            completeness: 0.7,
            consistency: 0.9,
            relevance_score: 0.0,
            missing_fields: Vec::new(),
        }
    }
}

impl Default for DataSource {
    fn default() -> Self {
        Self {
            primary_provider: AnimeProvider::AniList,
            providers_used: vec![AnimeProvider::AniList],
            confidence: 0.8,
            fetch_time_ms: 1000,
        }
    }
}

impl AnimeData {
    /// Create new AnimeData with default quality metrics
    pub fn new(anime: AnimeDetailed) -> Self {
        Self {
            anime,
            quality: DataQuality::default(),
            source: DataSource::default(),
        }
    }

    /// Create AnimeData with specific quality and source info
    pub fn with_metadata(anime: AnimeDetailed, quality: DataQuality, source: DataSource) -> Self {
        Self {
            anime,
            quality,
            source,
        }
    }

    /// Check if the anime data meets quality threshold
    pub fn meets_quality_threshold(&self) -> bool {
        self.quality.score >= 0.6 && self.quality.completeness >= 0.5
    }

    /// Get quality assessment as human-readable string
    pub fn quality_assessment(&self) -> &'static str {
        match self.quality.score {
            s if s >= 0.9 => "Excellent",
            s if s >= 0.8 => "Good",
            s if s >= 0.7 => "Fair",
            s if s >= 0.6 => "Poor",
            _ => "Very Poor",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::anime::domain::value_objects::{AnimeStatus, AnimeTitle, AnimeType};
    use crate::modules::provider::domain::value_objects::provider_metadata::ProviderMetadata;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_anime() -> AnimeDetailed {
        AnimeDetailed {
            id: Uuid::new_v4(),
            title: AnimeTitle {
                main: "Test Anime".to_string(),
                english: Some("Test Anime EN".to_string()),
                japanese: Some("テストアニメ".to_string()),
                romaji: Some("Test Anime".to_string()),
                native: Some("テストアニメ".to_string()),
                synonyms: vec![],
            },
            provider_metadata: ProviderMetadata::new(AnimeProvider::AniList, "12345".to_string()),
            score: Some(8.5),
            rating: Some(8.5),
            favorites: Some(1000),
            synopsis: Some("Test synopsis".to_string()),
            description: Some("Test description".to_string()),
            episodes: Some(12),
            status: AnimeStatus::Finished,
            aired: crate::modules::anime::domain::entities::anime_detailed::AiredDates {
                from: None,
                to: None,
            },
            anime_type: AnimeType::TV,
            age_restriction: None,
            genres: vec![],
            studios: vec![],
            source: None,
            duration: None,
            image_url: None,
            images: None,
            banner_image: None,
            trailer_url: None,
            composite_score: 8.5,
            tier: crate::modules::anime::domain::value_objects::AnimeTier::default(),
            quality_metrics: crate::modules::anime::domain::value_objects::QualityMetrics::default(
            ),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_synced_at: Some(Utc::now()),
        }
    }

    #[test]
    fn test_anime_data_creation() {
        let anime = create_test_anime();
        let anime_data = AnimeData::new(anime.clone());

        assert_eq!(anime_data.anime.title.main, "Test Anime");
        assert_eq!(anime_data.quality.score, 0.8);
        assert_eq!(anime_data.source.primary_provider, AnimeProvider::AniList);
    }

    #[test]
    fn test_anime_data_with_metadata() {
        let anime = create_test_anime();
        let quality = DataQuality {
            score: 0.95,
            completeness: 0.9,
            consistency: 0.95,
            relevance_score: 0.0,
            missing_fields: vec![],
        };
        let source = DataSource {
            primary_provider: AnimeProvider::Jikan,
            providers_used: vec![AnimeProvider::Jikan, AnimeProvider::AniList],
            confidence: 0.9,
            fetch_time_ms: 500,
        };

        let anime_data = AnimeData::with_metadata(anime, quality.clone(), source.clone());

        assert_eq!(anime_data.quality.score, 0.95);
        assert_eq!(anime_data.source.primary_provider, AnimeProvider::Jikan);
        assert_eq!(anime_data.source.providers_used.len(), 2);
    }
}
