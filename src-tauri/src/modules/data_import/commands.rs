use crate::modules::data_import::domain::services::import_components::{
    BatchQualityInsights, EnhancedValidationResult,
};
use crate::modules::data_import::{ImportResult, ImportService, ValidatedAnime};
use crate::{log_debug, log_info};
use serde::Deserialize;
use specta::Type;
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Deserialize, Type)]
pub struct ImportAnimeBatchRequest {
    pub titles: Vec<String>,
    // // TODO: Implement configuration options
    // pub enhance_data: Option<bool>,     // Controls data enhancement
    // pub fill_gaps: Option<bool>,        // Controls gap filling
    // pub quality_threshold: Option<f32>, // Sets minimum quality threshold
}

#[derive(Debug, Deserialize, Type)]
pub struct ValidateAnimeTitlesRequest {
    pub titles: Vec<String>,
}

#[derive(Debug, Deserialize, Type)]
pub struct ImportValidatedAnimeRequest {
    pub validated_anime: Vec<ValidatedAnime>,
}

#[derive(Debug, serde::Serialize, Type)]
pub struct ImportBatchResult {
    pub imported_anime: Vec<ImportResult>,
    pub quality_insights: BatchQualityInsights,
    pub providers_used: Vec<String>,
    pub gaps_filled: u32,
}

#[tauri::command]
#[specta::specta]
pub async fn import_anime_batch(
    request: ImportAnimeBatchRequest,
    import_service: State<'_, Arc<ImportService>>,
    app_handle: tauri::AppHandle,
) -> Result<ImportBatchResult, String> {
    log_debug!(
        "import_anime_batch command called with {} titles",
        request.titles.len()
    );

    let result = import_service
        .import_anime_batch_enhanced(request.titles, Some(app_handle), None)
        .await
        .map_err(|e| e.to_string());

    match &result {
        Ok((import_result, quality_insights)) => {
            log_info!(
                "Enhanced import completed - Imported: {}, Quality Score: {:.1}",
                import_result.imported.len(),
                quality_insights.average_quality_after
            );

            // Transform result to enhanced format
            Ok(ImportBatchResult {
                imported_anime: vec![import_result.clone()],
                quality_insights: quality_insights.clone(),
                providers_used: vec![
                    "AniList".to_string(),
                    "MyAnimeList".to_string(),
                    "Jikan".to_string(),
                ], // TODO: Get from actual providers used
                gaps_filled: quality_insights.common_gaps.values().sum(),
            })
        }
        Err(e) => {
            log_debug!("Enhanced import failed with error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn validate_anime_titles(
    request: ValidateAnimeTitlesRequest,
    import_service: State<'_, Arc<ImportService>>,
    app_handle: tauri::AppHandle,
) -> Result<EnhancedValidationResult, String> {
    log_debug!(
        "validate_anime_titles command called with {} titles",
        request.titles.len()
    );

    let result = import_service
        .validate_anime_titles_enhanced(request.titles, Some(&app_handle))
        .await
        .map_err(|e| e.to_string());

    match &result {
        Ok(enhanced_result) => {
            log_info!(
                "Enhanced validation completed - Found: {}, Confidence: {:.1}%",
                enhanced_result.found.len(),
                enhanced_result.average_confidence
            );
        }
        Err(e) => {
            log_debug!("Enhanced validation failed with error: {}", e);
        }
    }

    result
}

#[tauri::command]
#[specta::specta]
pub async fn import_validated_anime(
    request: ImportValidatedAnimeRequest,
    import_service: State<'_, Arc<ImportService>>,
    app_handle: tauri::AppHandle,
) -> Result<ImportResult, String> {
    log_debug!(
        "import_validated_anime command called with {} anime",
        request.validated_anime.len()
    );
    let result = import_service
        .import_validated_anime(request.validated_anime, Some(app_handle))
        .await
        .map_err(|e| e.to_string());

    match &result {
        Ok(import_result) => {
            log_info!(
                "Import completed - Imported: {}, Skipped: {}, Failed: {}",
                import_result.imported.len(),
                import_result.skipped.len(),
                import_result.failed.len()
            );
        }
        Err(e) => {
            log_debug!("Import failed with error: {}", e);
        }
    }

    result
}
