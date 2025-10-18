use super::genre::Genre;
use crate::modules::anime::domain::value_objects::{
    AnimeStatus, AnimeTier, AnimeTitle, AnimeType, QualityMetrics,
};
use crate::shared::domain::value_objects::{
    AnimeProvider, ProviderMetadata, UnifiedAgeRestriction,
};
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

    // ============================================================================================
    // VALIDATION METHODS (Aggregate Boundary Protection)
    // ============================================================================================

    /// Update score with validation (must be 0-10)
    pub fn update_score(&mut self, score: f32) -> Result<(), String> {
        if score < 0.0 || score > 10.0 {
            return Err(format!("Score must be between 0-10, got {}", score));
        }
        self.score = Some(score);
        self.rating = Some(score); // Keep rating in sync
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update episodes with validation (must be positive)
    pub fn update_episodes(&mut self, episodes: u16) -> Result<(), String> {
        if episodes == 0 {
            return Err("Episodes must be greater than 0".to_string());
        }
        self.episodes = Some(episodes);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update title with validation (cannot be empty)
    pub fn update_title(&mut self, title: AnimeTitle) -> Result<(), String> {
        if title.main.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        self.title = title;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update composite score and tier together (maintains invariant)
    pub fn update_composite_score_and_tier(
        &mut self,
        composite_score: f32,
        tier: AnimeTier,
    ) -> Result<(), String> {
        if composite_score < 0.0 || composite_score > 10.0 {
            return Err(format!(
                "Composite score must be between 0-10, got {}",
                composite_score
            ));
        }
        self.composite_score = composite_score;
        self.tier = tier;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Validate current state
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.title.main.trim().is_empty() {
            errors.push("Title cannot be empty".to_string());
        }

        if let Some(score) = self.score {
            if score < 0.0 || score > 10.0 {
                errors.push(format!("Score must be 0-10, got {}", score));
            }
        }

        if let Some(episodes) = self.episodes {
            if episodes == 0 {
                errors.push("Episodes must be greater than 0".to_string());
            }
        }

        if self.composite_score < 0.0 || self.composite_score > 10.0 {
            errors.push(format!(
                "Composite score must be 0-10, got {}",
                self.composite_score
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    // ============================================================================================
    // SAFE ACCESSORS (Encapsulation Support)
    // ============================================================================================

    /// Get ID (immutable)
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get title (immutable reference)
    pub fn title(&self) -> &AnimeTitle {
        &self.title
    }

    /// Get score (safe copy)
    pub fn score(&self) -> Option<f32> {
        self.score
    }

    /// Get composite score
    pub fn composite_score(&self) -> f32 {
        self.composite_score
    }

    /// Get tier
    pub fn tier(&self) -> AnimeTier {
        self.tier
    }
}
