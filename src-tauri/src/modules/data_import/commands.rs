use crate::modules::data_import::{ImportResult, ImportService, ValidatedAnime, ValidationResult};
use crate::{log_debug, log_info};
use serde::Deserialize;
use specta::Type;
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Deserialize, Type)]
pub struct ImportAnimeBatchRequest {
    pub titles: Vec<String>,
}

#[derive(Debug, Deserialize, Type)]
pub struct ValidateAnimeTitlesRequest {
    pub titles: Vec<String>,
}

#[derive(Debug, Deserialize, Type)]
pub struct ImportValidatedAnimeRequest {
    pub validated_anime: Vec<ValidatedAnime>,
}

#[tauri::command]
#[specta::specta]
pub async fn import_anime_batch(
    request: ImportAnimeBatchRequest,
    import_service: State<'_, Arc<ImportService>>,
    app_handle: tauri::AppHandle,
) -> Result<ImportResult, String> {
    import_service
        .import_anime_batch(request.titles, Some(app_handle), None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn validate_anime_titles(
    request: ValidateAnimeTitlesRequest,
    import_service: State<'_, Arc<ImportService>>,
    app_handle: tauri::AppHandle,
) -> Result<ValidationResult, String> {
    import_service
        .validate_anime_titles(request.titles, Some(&app_handle))
        .await
        .map_err(|e| e.to_string())
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
