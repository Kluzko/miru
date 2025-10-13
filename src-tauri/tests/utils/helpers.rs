/// Test helper functions and service builders
use super::db::get_test_db_pool;
use miru_lib::modules::{
    anime::{
        application::{ingestion_service::AnimeIngestionService, service::AnimeService},
        infrastructure::persistence::AnimeRepositoryImpl,
        AnimeRepository,
    },
    data_import::domain::services::import_components::{
        data_enhancement_service::DataEnhancementService, validation_service::ValidationService,
    },
    jobs::{infrastructure::JobRepositoryImpl, worker::BackgroundWorker},
    provider::{
        application::service::ProviderService,
        infrastructure::adapters::{CacheAdapter, ProviderRepositoryAdapter},
    },
};
use miru_lib::shared::database::Database;
use std::sync::Arc;

pub struct TestServices {
    pub ingestion_service: Arc<AnimeIngestionService>,
    pub anime_service: Arc<AnimeService>,
    pub job_repository: Arc<JobRepositoryImpl>,
    pub background_worker: Arc<BackgroundWorker>,
    pub anime_repository: Arc<dyn AnimeRepository>,
}

/// Build all services needed for integration tests
pub fn build_test_services() -> TestServices {
    let pool = super::db::get_test_db_pool();
    // Use the singleton test pool instead of creating a new Database
    let db = Arc::new(Database::from_pool((*pool).clone()));

    let anime_repo: Arc<dyn AnimeRepository> = Arc::new(AnimeRepositoryImpl::new(db.clone()));
    let job_repo = Arc::new(JobRepositoryImpl::new((*pool).clone()));

    let provider_repo = Arc::new(ProviderRepositoryAdapter::new());
    let cache_repo = Arc::new(CacheAdapter::new());
    let provider_service = Arc::new(ProviderService::new(provider_repo, cache_repo));

    let anime_service = Arc::new(AnimeService::new(
        anime_repo.clone(),
        provider_service.clone(),
    ));

    let validation_service = Arc::new(ValidationService::new(
        anime_repo.clone(),
        provider_service.clone(),
    ));
    let enhancement_service = Arc::new(DataEnhancementService::new(provider_service.clone()));

    let ingestion_service = Arc::new(AnimeIngestionService::new(
        validation_service,
        enhancement_service,
        anime_service.clone(),
        provider_service.clone(),
        job_repo.clone(),
    ));

    let background_worker = Arc::new(BackgroundWorker::new(
        job_repo.clone(),
        anime_service.clone(),
        provider_service.clone(),
    ));

    TestServices {
        ingestion_service,
        anime_service,
        job_repository: job_repo,
        background_worker,
        anime_repository: anime_repo,
    }
}

/// Run background worker for a specified duration then stop it
pub async fn run_worker_for_duration(worker: Arc<BackgroundWorker>, duration_secs: u64) {
    let handle = worker.clone().start();
    tokio::time::sleep(tokio::time::Duration::from_secs(duration_secs)).await;
    worker.stop().await;
    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(2), handle).await;
}
