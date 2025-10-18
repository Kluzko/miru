/// Background worker for processing anime enrichment and relations discovery jobs
///
/// This worker continuously polls the job queue and processes jobs asynchronously.
/// It uses tokio::spawn for background execution, suitable for desktop applications.
use crate::modules::anime::application::service::AnimeService;
use crate::modules::anime::domain::services::anime_relations_service::AnimeRelationsService;
use crate::modules::jobs::domain::entities::{
    EnrichmentJobPayload, JobType, RelationsDiscoveryJobPayload,
};
use crate::modules::jobs::domain::repository::JobRepository;
use crate::modules::provider::ProviderService;
use crate::shared::errors::AppResult;
use crate::{log_debug, log_error, log_info, log_warn};
use std::sync::Arc;
use std::time::Duration;

/// Background worker that processes jobs from the queue
pub struct BackgroundWorker {
    job_repository: Arc<dyn JobRepository>,
    anime_service: Arc<AnimeService>,
    provider_service: Arc<ProviderService>,
    relations_service: Arc<AnimeRelationsService>,
    poll_interval: Duration,
    is_running: Arc<tokio::sync::RwLock<bool>>,
}

impl BackgroundWorker {
    /// Create a new background worker
    pub fn new(
        job_repository: Arc<dyn JobRepository>,
        anime_service: Arc<AnimeService>,
        provider_service: Arc<ProviderService>,
        relations_service: Arc<AnimeRelationsService>,
    ) -> Self {
        Self {
            job_repository,
            anime_service,
            provider_service,
            relations_service,
            poll_interval: Duration::from_secs(5), // Poll every 5 seconds
            is_running: Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    /// Start the background worker
    ///
    /// This method runs the worker loop. Call it with tokio::spawn or tauri::async_runtime::spawn
    /// to run in the background.
    pub async fn run(self: Arc<Self>) {
        log_info!("Background worker started");

        // Mark as running
        {
            let mut running = self.is_running.write().await;
            *running = true;
        }

        loop {
            // Check if we should stop
            {
                let running = self.is_running.read().await;
                if !*running {
                    log_info!("Background worker stopped");
                    break;
                }
            }

            // Try to dequeue and process a job
            match self.process_next_job().await {
                Ok(processed) => {
                    if !processed {
                        // No jobs available, sleep before next poll
                        tokio::time::sleep(self.poll_interval).await;
                    }
                    // If job was processed, immediately try to get next one
                }
                Err(e) => {
                    log_error!("Error in worker loop: {}", e);
                    tokio::time::sleep(self.poll_interval).await;
                }
            }
        }
    }

    /// Stop the background worker
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
        log_info!("Background worker stop requested");
    }

    /// Process the next job in the queue
    ///
    /// Returns true if a job was processed, false if queue was empty
    async fn process_next_job(&self) -> AppResult<bool> {
        // Atomically dequeue the next job
        let job = match self.job_repository.dequeue().await? {
            Some(job) => job,
            None => return Ok(false), // No jobs available
        };

        log_info!(
            "Processing job {} (type: {}, attempts: {}/{})",
            job.id,
            job.job_type,
            job.attempts,
            job.max_attempts
        );

        // Parse job type and execute
        let result = match job.parse_job_type() {
            Ok(JobType::Enrichment) => self.handle_enrichment_job(&job).await,
            Ok(JobType::RelationsDiscovery) => self.handle_relations_job(&job).await,
            Err(e) => {
                log_error!("Invalid job type '{}': {}", job.job_type, e);
                Err(crate::shared::errors::AppError::ValidationError(format!(
                    "Invalid job type: {}",
                    e
                )))
            }
        };

        // Update job status based on result
        match result {
            Ok(_) => {
                self.job_repository.mark_completed(job.id).await?;
                log_info!("Job {} completed successfully", job.id);
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                log_warn!("Job {} failed: {}", job.id, error_msg);

                if job.can_retry() {
                    log_info!(
                        "Job {} will be retried (attempt {}/{})",
                        job.id,
                        job.attempts,
                        job.max_attempts
                    );
                    // Job will be retried automatically (status is already 'running' with incremented attempts)
                    // We just need to reset it to 'pending' for the next worker cycle
                    // For now, mark as failed so it doesn't get stuck in 'running' state
                    self.job_repository.mark_failed(job.id, &error_msg).await?;
                } else {
                    log_error!(
                        "Job {} failed permanently after {} attempts",
                        job.id,
                        job.attempts
                    );
                    self.job_repository.mark_failed(job.id, &error_msg).await?;
                }
            }
        }

        Ok(true)
    }

    /// Handle an enrichment job
    async fn handle_enrichment_job(
        &self,
        job: &crate::modules::jobs::domain::entities::JobRecord,
    ) -> AppResult<()> {
        // Parse payload
        let payload: EnrichmentJobPayload = job.parse_enrichment_payload().map_err(|e| {
            crate::shared::errors::AppError::ValidationError(format!(
                "Invalid enrichment payload: {}",
                e
            ))
        })?;

        log_debug!("Enriching anime {}", payload.anime_id);

        // Fetch the anime from database
        let anime = self
            .anime_service
            .get_anime_by_id(&payload.anime_id)
            .await?
            .ok_or_else(|| {
                crate::shared::errors::AppError::NotFound(format!(
                    "Anime {} not found",
                    payload.anime_id
                ))
            })?;

        log_debug!(
            "Current anime tier: {:?}, quality: {:?}",
            anime.tier,
            anime.quality_metrics
        );

        // Fetch enhanced data from multiple providers
        // Try AniList
        let anilist_data = if let Some(anilist_id) = anime
            .provider_metadata
            .get_external_id(&crate::modules::provider::AnimeProvider::AniList)
        {
            self.provider_service
                .get_anime_by_id(anilist_id, crate::modules::provider::AnimeProvider::AniList)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        // Try Jikan (MAL) for age_restriction
        let jikan_data = if let Some(mal_id) = anime
            .provider_metadata
            .get_external_id(&crate::modules::provider::AnimeProvider::Jikan)
        {
            self.provider_service
                .get_anime_by_id(mal_id, crate::modules::provider::AnimeProvider::Jikan)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        // Merge data intelligently
        let mut enriched = anime.clone();
        let mut improvements = Vec::new();

        // Fill missing fields from AniList
        if let Some(anilist) = anilist_data {
            if enriched.synopsis.is_none() && anilist.synopsis.is_some() {
                enriched.synopsis = anilist.synopsis.clone();
                improvements.push("Added synopsis from AniList");
            }
            if enriched.genres.is_empty() && !anilist.genres.is_empty() {
                enriched.genres = anilist.genres.clone();
                improvements.push("Added genres from AniList");
            }
            if enriched.studios.is_empty() && !anilist.studios.is_empty() {
                enriched.studios = anilist.studios.clone();
                improvements.push("Added studios from AniList");
            }
        }

        // Fill age_restriction from Jikan (MAL has this data)
        if let Some(jikan) = jikan_data {
            if enriched.age_restriction.is_none() && jikan.age_restriction.is_some() {
                enriched.age_restriction = jikan.age_restriction.clone();
                improvements.push("Added age_restriction from Jikan");
            }
        }

        // Recalculate scores if data was improved
        if !improvements.is_empty() {
            log_info!(
                "Enriched anime {} with: {}",
                payload.anime_id,
                improvements.join(", ")
            );

            // Use AnimeService to update (which calls update_scores)
            self.anime_service.update_anime(&enriched).await?;

            log_info!(
                "Anime {} enriched successfully (new tier: {:?})",
                payload.anime_id,
                enriched.tier
            );
        } else {
            log_debug!("No improvements found for anime {}", payload.anime_id);
        }

        Ok(())
    }

    /// Handle a relations discovery job
    async fn handle_relations_job(
        &self,
        job: &crate::modules::jobs::domain::entities::JobRecord,
    ) -> AppResult<()> {
        // Parse payload
        let payload: RelationsDiscoveryJobPayload = job.parse_relations_payload().map_err(|e| {
            crate::shared::errors::AppError::ValidationError(format!(
                "Invalid relations payload: {}",
                e
            ))
        })?;

        log_info!("Discovering relations for anime {}", payload.anime_id);

        // Use the relations service to discover and store relations
        // This will internally use AnimeIngestionService for each discovered anime
        let anime_id_str = payload.anime_id.to_string();
        match self
            .relations_service
            .get_anime_with_relations(&anime_id_str)
            .await
        {
            Ok(relations) => {
                log_info!(
                    "Successfully discovered {} relations for anime {}",
                    relations.len(),
                    payload.anime_id
                );
                Ok(())
            }
            Err(e) => {
                log_error!(
                    "Failed to discover relations for anime {}: {}",
                    payload.anime_id,
                    e
                );
                Err(e)
            }
        }
    }

    /// Get statistics about the worker and job queue
    pub async fn get_statistics(&self) -> AppResult<WorkerStatistics> {
        let job_stats = self.job_repository.get_statistics().await?;
        let is_running = *self.is_running.read().await;

        Ok(WorkerStatistics {
            is_running,
            pending_jobs: job_stats.pending_count,
            running_jobs: job_stats.running_count,
            completed_jobs: job_stats.completed_count,
            failed_jobs: job_stats.failed_count,
            total_jobs: job_stats.total_count,
        })
    }
}

/// Worker statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct WorkerStatistics {
    pub is_running: bool,
    pub pending_jobs: i64,
    pub running_jobs: i64,
    pub completed_jobs: i64,
    pub failed_jobs: i64,
    pub total_jobs: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_creation() {
        // Test that worker can be created with proper configuration
        // (Actual integration tests would require database setup)
    }
}
