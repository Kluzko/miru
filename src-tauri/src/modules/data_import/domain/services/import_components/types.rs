use specta::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ImportResult {
    pub imported: Vec<ImportedAnime>,
    pub failed: Vec<ImportError>,
    pub skipped: Vec<SkippedAnime>,
    pub total: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ImportedAnime {
    pub title: String,
    pub primary_external_id: String,
    pub provider: crate::modules::provider::AnimeProvider,
    pub id: uuid::Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ImportError {
    pub title: String,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct SkippedAnime {
    pub title: String,
    pub external_id: String,
    pub provider: crate::modules::provider::AnimeProvider,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ValidationResult {
    pub found: Vec<ValidatedAnime>,
    pub not_found: Vec<ImportError>,
    pub already_exists: Vec<ExistingAnime>,
    pub total: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ValidatedAnime {
    pub input_title: String,
    pub anime_data: crate::modules::anime::AnimeDetailed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ExistingAnime {
    pub input_title: String,
    pub matched_title: String,
    pub matched_field: String,
    pub anime: crate::modules::anime::AnimeDetailed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ImportProgress {
    pub current: usize,
    pub total: usize,
    pub current_title: String,
    pub processed: usize,
    pub imported_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ValidationProgress {
    pub current: usize,
    pub total: usize,
    pub current_title: String,
    pub processed: usize,
    pub found_count: usize,
    pub existing_count: usize,
    pub failed_count: usize,
}

// ========================================================================
// ENHANCED TYPES FOR COMPREHENSIVE IMPORT
// ========================================================================

/// Enhanced validated anime with comprehensive data from multiple providers
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct EnhancedValidatedAnime {
    pub input_title: String,
    pub anime_data: crate::modules::anime::AnimeDetailed,
    pub data_quality: DataQualityMetrics,
    pub provider_sources: Vec<crate::modules::provider::AnimeProvider>,
    pub confidence_score: f32, // 0.0 to 1.0
}

/// Data quality metrics for imported anime
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct DataQualityMetrics {
    pub completeness_score: f32, // Percentage of fields that are filled
    pub consistency_score: f32,  // How consistent data is across providers
    pub freshness_score: f32,    // How recent the data is
    pub source_reliability: f32, // Reliability of primary data source
    pub field_completeness: HashMap<String, bool>, // Which fields are complete
    pub provider_agreements: HashMap<String, usize>, // How many providers agree on each field
}

/// Enhanced validation result with comprehensive data analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct EnhancedValidationResult {
    pub found: Vec<EnhancedValidatedAnime>,
    pub not_found: Vec<ImportError>,
    pub already_exists: Vec<ExistingAnime>,
    pub total: u32,
    pub average_confidence: f32,
    pub data_quality_summary: DataQualitySummary,
}

/// Summary of data quality across all validated anime
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct DataQualitySummary {
    pub average_completeness: f32,
    pub average_consistency: f32,
    pub total_providers_used: usize,
    pub most_reliable_provider: Option<crate::modules::provider::AnimeProvider>,
    pub fields_with_gaps: Vec<String>, // Fields that are commonly missing
}

/// Enhanced import result with data provenance tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct EnhancedImportResult {
    pub imported: Vec<EnhancedImportedAnime>,
    pub failed: Vec<ImportError>,
    pub skipped: Vec<SkippedAnime>,
    pub total: u32,
    pub data_enhancement_stats: DataEnhancementStats,
}

/// Enhanced imported anime with provenance and quality info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct EnhancedImportedAnime {
    pub title: String,
    pub primary_external_id: String,
    pub provider: crate::modules::provider::AnimeProvider,
    pub id: uuid::Uuid,
    pub data_sources: Vec<crate::modules::provider::AnimeProvider>,
    pub enhancement_applied: bool,
    pub fields_enhanced: Vec<String>,
    pub final_confidence: f32,
}

/// Statistics about data enhancement during import
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct DataEnhancementStats {
    pub anime_enhanced: usize,
    pub total_fields_filled: usize,
    pub average_sources_per_anime: f32,
    pub most_enhanced_fields: Vec<String>,
    pub enhancement_success_rate: f32,
}
