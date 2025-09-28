use super::genre::Genre;
use crate::modules::anime::domain::value_objects::{
    AnimeStatus, AnimeTier, AnimeTitle, AnimeType, QualityMetrics,
};
use crate::modules::provider::domain::{AnimeProvider, ProviderMetadata};
use crate::shared::domain::value_objects::UnifiedAgeRestriction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

mod scoring;

// ================================================================================================
// HELPER TYPES
// ================================================================================================

/// Air date range for anime
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AiredDates {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

// ================================================================================================
// MAIN ANIME DETAILED ENTITY
// ================================================================================================

/// Comprehensive anime entity with full information for detailed views
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AnimeDetailed {
    // Core identification
    pub id: Uuid,
    pub title: AnimeTitle,

    // Provider metadata (external IDs, sync info, etc.)
    pub provider_metadata: ProviderMetadata,

    // Scoring information (unified across providers, 0-10 scale)
    pub score: Option<f32>,
    pub rating: Option<f32>,    // Alias for score
    pub favorites: Option<u32>, // Used as engagement metric

    // Content information
    pub synopsis: Option<String>,
    pub description: Option<String>, // Alias for synopsis
    pub episodes: Option<u16>,
    pub status: AnimeStatus,
    pub aired: AiredDates,
    pub anime_type: AnimeType,

    // Age restriction (mapped from provider data during ingestion)
    pub age_restriction: Option<UnifiedAgeRestriction>,

    // Classifications and metadata
    pub genres: Vec<Genre>,
    pub studios: Vec<String>,
    pub source: Option<String>,
    pub duration: Option<String>,

    // Media content
    pub image_url: Option<String>,
    pub images: Option<String>, // Alias for image_url
    pub banner_image: Option<String>,
    pub trailer_url: Option<String>,

    // Internal scoring system
    pub composite_score: f32,
    pub tier: AnimeTier,
    pub quality_metrics: QualityMetrics,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_synced_at: Option<DateTime<Utc>>, // For tracking provider resync
}

// ================================================================================================
// TRAIT IMPLEMENTATIONS
// ================================================================================================
// Scoreable trait implementation is in scoring.rs module

// ================================================================================================
// CORE IMPLEMENTATION
// ================================================================================================

impl AnimeDetailed {
    /// Create new anime with minimal required data
    pub fn new(primary_provider: AnimeProvider, external_id: String, title: String) -> Self {
        let id = Uuid::new_v4();
        let now = Utc::now();

        Self {
            id,
            title: AnimeTitle::new(title),
            provider_metadata: ProviderMetadata::new(primary_provider, external_id),
            score: None,
            rating: None,
            favorites: None,
            synopsis: None,
            description: None,
            episodes: None,
            status: AnimeStatus::Unknown,
            aired: AiredDates {
                from: None,
                to: None,
            },
            anime_type: AnimeType::Unknown,
            age_restriction: None,
            genres: Vec::new(),
            studios: Vec::new(),
            source: None,
            duration: None,
            image_url: None,
            images: None,
            banner_image: None,
            trailer_url: None,
            composite_score: 0.0,
            tier: AnimeTier::default(),
            quality_metrics: QualityMetrics::default(),
            created_at: now,
            updated_at: now,
            last_synced_at: None,
        }
    }
}
