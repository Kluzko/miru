use crate::modules::provider::domain::entities::anime_data::AnimeData;
use crate::modules::provider::AnimeProvider;

/// Context for merging anime data
/// Contains all information needed to make intelligent merge decisions
#[derive(Debug, Clone)]
pub struct MergeContext {
    /// Base anime data (highest quality)
    pub base: AnimeData,

    /// Additional data sources to merge from
    pub sources: Vec<AnimeData>,

    /// Provider-specific preferences
    pub provider_preferences: ProviderPreferences,
}

/// Preferences for which providers to trust for specific fields
#[derive(Debug, Clone)]
pub struct ProviderPreferences {
    /// Provider to prefer for age ratings (typically Jikan/MAL)
    pub age_rating_provider: Option<AnimeProvider>,

    /// Provider to prefer for images (typically AniList)
    pub image_provider: Option<AnimeProvider>,

    /// Provider to prefer for metadata (descriptions, titles)
    pub metadata_provider: Option<AnimeProvider>,
}

impl Default for ProviderPreferences {
    fn default() -> Self {
        use crate::modules::provider::AnimeProvider;
        Self {
            age_rating_provider: Some(AnimeProvider::Jikan),
            image_provider: Some(AnimeProvider::AniList),
            metadata_provider: None, // No preference, use quality-based selection
        }
    }
}

impl MergeContext {
    pub fn new(base: AnimeData, sources: Vec<AnimeData>) -> Self {
        Self {
            base,
            sources,
            provider_preferences: ProviderPreferences::default(),
        }
    }

    pub fn with_preferences(mut self, preferences: ProviderPreferences) -> Self {
        self.provider_preferences = preferences;
        self
    }

    /// Get data from preferred provider for a specific field type
    pub fn get_from_preferred_provider(
        &self,
        preferred_provider: Option<AnimeProvider>,
    ) -> Option<&AnimeData> {
        if let Some(provider) = preferred_provider {
            self.sources
                .iter()
                .find(|data| data.source.primary_provider == provider)
        } else {
            None
        }
    }
}
