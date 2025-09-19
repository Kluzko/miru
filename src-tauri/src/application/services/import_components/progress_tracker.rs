use crate::log_error;
use std::sync::Arc;
use tauri::Emitter;

use super::types::{ImportProgress, ValidationProgress};

/// Manages progress reporting and batching for import operations
#[derive(Clone)]
pub struct ProgressTracker {
    app_handle: Option<Arc<tauri::AppHandle>>,
    batch_config: ProgressBatchConfig,
}

#[derive(Clone)]
struct ProgressBatchConfig {
    batch_size: usize,
    min_percentage_change: usize,
}

impl Default for ProgressBatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 1,
            min_percentage_change: 1,
        }
    }
}

impl ProgressTracker {
    pub fn new(app_handle: Option<tauri::AppHandle>) -> Self {
        Self {
            app_handle: app_handle.map(Arc::new),
            batch_config: ProgressBatchConfig::default(),
        }
    }

    pub fn with_batch_config(mut self, total_items: usize) -> Self {
        self.batch_config.batch_size = std::cmp::max(1, total_items / 50);
        self
    }

    pub fn emit_import_progress(&self, progress: ImportProgress) -> bool {
        if let Some(ref app) = self.app_handle {
            match app.emit("import_progress", &progress) {
                Ok(_) => true,
                Err(e) => {
                    log_error!("Failed to emit import progress: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn emit_validation_progress(&self, progress: ValidationProgress) -> bool {
        if let Some(ref app) = self.app_handle {
            match app.emit("validation_progress", &progress) {
                Ok(_) => true,
                Err(e) => {
                    log_error!("Failed to emit validation progress: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Helper for batched validation progress with existing logic
    pub fn should_emit_validation_progress(
        &self,
        processed: usize,
        total: usize,
        last_emitted_percentage: &mut usize,
        is_initial: bool,
        is_final: bool,
    ) -> bool {
        let current_percentage = if total > 0 {
            (processed * 100) / total
        } else {
            0
        };
        let should_emit_percentage = current_percentage.saturating_sub(*last_emitted_percentage)
            >= self.batch_config.min_percentage_change;
        let should_emit_batch = processed % self.batch_config.batch_size == 0;

        let should_emit = is_initial || is_final || should_emit_percentage || should_emit_batch;

        if should_emit {
            *last_emitted_percentage = current_percentage;
        }

        should_emit
    }
}
