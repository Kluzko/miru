use specta::Type;

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
    pub provider: crate::domain::value_objects::AnimeProvider,
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
    pub provider: crate::domain::value_objects::AnimeProvider,
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
    pub anime_data: crate::domain::entities::AnimeDetailed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct ExistingAnime {
    pub input_title: String,
    pub matched_title: String,
    pub matched_field: String,
    pub anime: crate::domain::entities::AnimeDetailed,
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
