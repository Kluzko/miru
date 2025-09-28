use crate::modules::provider::domain::entities::anime_data::AnimeData;
use crate::shared::errors::AppError;

/// Main mapper trait for converting provider-specific data to domain AnimeData
pub trait AnimeMapper<T> {
    /// Map provider data to domain AnimeData
    fn map_to_anime_data(&self, source: T) -> Result<AnimeData, AppError>;

    /// Map a list of provider data to domain AnimeData
    fn map_to_anime_data_list(&self, sources: Vec<T>) -> Result<Vec<AnimeData>, AppError> {
        sources
            .into_iter()
            .map(|source| self.map_to_anime_data(source))
            .collect()
    }
}

/// Capability trait to describe what each adapter can provide
pub trait AdapterCapabilities {
    /// Get the name of the adapter
    fn name(&self) -> &'static str;

    /// Get the provider fields this adapter can populate
    fn supported_fields(&self) -> Vec<&'static str>;

    /// Get the provider fields this adapter cannot populate
    fn unsupported_fields(&self) -> Vec<&'static str>;

    /// Check if the adapter supports a specific field
    fn supports_field(&self, field: &str) -> bool {
        self.supported_fields().contains(&field)
    }

    /// Get quality score for this adapter (0.0 to 1.0)
    fn quality_score(&self) -> f64;

    /// Get response time estimate in milliseconds
    fn estimated_response_time(&self) -> u64;

    /// Check if the adapter has rate limiting
    fn has_rate_limiting(&self) -> bool;
}

/// Provider identification for mappers
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderId {
    AniList,
    Jikan,
}

impl ProviderId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderId::AniList => "anilist",
            ProviderId::Jikan => "jikan",
        }
    }
}

/// Mapper registry trait for managing multiple mappers
pub trait MapperRegistry {
    /// Get all available mappers
    fn get_mappers(&self) -> Vec<Box<dyn AdapterCapabilities>>;

    /// Get mapper by provider ID
    fn get_mapper(&self, provider: ProviderId) -> Option<Box<dyn AdapterCapabilities>>;

    /// Get the best mapper for a specific field
    fn get_best_mapper_for_field(&self, field: &str) -> Option<Box<dyn AdapterCapabilities>>;
}
