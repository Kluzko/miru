pub mod commands;
pub mod modules;
mod schema;
pub mod shared;

use commands::get_all_commands;
use modules::{
    anime::{
        application::{ingestion_service::AnimeIngestionService, service::AnimeService},
        domain::services::anime_relations_service::{AnimeRelationsService, RelationsCache},
        infrastructure::persistence::{AnimeRelationsRepositoryImpl, AnimeRepositoryImpl},
        AnimeRepository,
    },
    collection::{
        application::service::CollectionService,
        infrastructure::persistence::CollectionRepositoryImpl, CollectionRepository,
    },
    data_import::{
        application::service::ImportService,
        domain::services::import_components::{
            data_enhancement_service::DataEnhancementService, validation_service::ValidationService,
        },
    },
    jobs::{infrastructure::JobRepositoryImpl, worker::BackgroundWorker},
    media::{
        application::{MediaService, MediaSyncService},
        infrastructure::{AnimeImageRepositoryImpl, AnimeVideoRepositoryImpl},
        AnimeImageRepository, AnimeVideoRepository,
    },
    provider::{
        application::service::ProviderService,
        domain::repositories::{
            AnimeProviderRepository, CacheRepository, MediaProviderRepository,
            RelationshipProviderRepository,
        },
        infrastructure::{
            adapters::{CacheAdapter, ProviderRepositoryAdapter},
            CachingRepositoryDecorator,
        },
    },
};
use shared::{DatabaseHealthMonitor, DatabaseState};
use std::sync::Arc;
use tauri::Manager;

// tauri-specta: generate TS types + typed command client from Rust commands
use specta_typescript::Typescript;
use tauri_specta::Builder as SpectaBuilder;

use tauri::async_runtime::{block_on, spawn};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load environment variables
    dotenvy::dotenv().ok();

    let specta_builder = SpectaBuilder::<tauri::Wry>::new().commands(get_all_commands());

    #[cfg(debug_assertions)]
    if let Err(e) = specta_builder.export(Typescript::default(), "../src/types/bindings.ts") {
        eprintln!("Warning: Failed to export TypeScript bindings: {}", e);
        eprintln!("TypeScript types may be out of sync. Consider running cargo build again.");
    }

    tauri::Builder::default()
        // Tell Tauri how to invoke commands from centralized registry
        .invoke_handler(crate::generate_handler_list!())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .clear_targets()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                ])
                .level(log::LevelFilter::Debug)
                .format(|out, message, record| {
                    // Extract meaningful module name from path
                    let target = record.target();

                    if target.starts_with("miru_lib::modules::") {
                        // Backend module: miru_lib::modules::anime::commands -> [LEVEL] [BACKEND] [anime] message
                        let module = target.strip_prefix("miru_lib::modules::")
                            .and_then(|s| s.split("::").next())
                            .unwrap_or("");
                        out.finish(format_args!(
                            "[{}] [BACKEND] [{}] {}",
                            record.level(),
                            module,
                            message
                        ))
                    } else if target.starts_with("miru_lib") {
                        // General backend: [LEVEL] [BACKEND] message
                        out.finish(format_args!(
                            "[{}] [BACKEND] {}",
                            record.level(),
                            message
                        ))
                    } else if target.starts_with("webview:") {
                        // Frontend logs: strip webview prefix, message already contains [FRONTEND] [module]
                        out.finish(format_args!(
                            "[{}] {}",
                            record.level(),
                            message
                        ))
                    } else {
                        // Other logs
                        out.finish(format_args!(
                            "[{}] [{}] {}",
                            record.level(),
                            target,
                            message
                        ))
                    }
                })
                .build(),
        )
        .setup(move |app| {
            // If you want typed events, mount specta's event hooks here.
            // `specta_builder` is moved into this closure (no later uses outside).
            specta_builder.mount_events(app);

            let db_state = DatabaseState::initialize();
            let health_monitor = DatabaseHealthMonitor::new(db_state);

            // Start background database health monitoring
            let monitor_state = health_monitor.get_state();

            spawn(async move {
                health_monitor.start_monitoring().await;
            });

            // Get database state for both migrations and service initialization
            let db_state_read = block_on(async {
                monitor_state.read().await.clone()
            });

            // Run migrations if database is available, otherwise continue with degraded functionality
            {
                use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
                const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

                match db_state_read.get_database() {
                    Ok(database) => {
                        match database.get_connection() {
                            Ok(mut conn) => {
                                if let Err(e) = conn.run_pending_migrations(MIGRATIONS) {
                                    log::error!("Failed to run database migrations: {}", e);
                                    log::warn!("Application will continue with limited functionality");
                                } else {
                                    log::info!("Database migrations completed successfully");
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to get database connection for migrations: {}", e);
                                log::warn!("Application will continue with limited functionality");
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Database unavailable during startup: {}", e);
                        log::warn!("Application will continue with limited functionality - database operations will be retried automatically");
                    }
                }
            }

            let provider_repo = Arc::new(ProviderRepositoryAdapter::new());
            let cache_repo = Arc::new(CacheAdapter::new());

            // Cast to trait objects for dependency injection
            // ProviderRepositoryAdapter implements both AnimeProviderRepository and MediaProviderRepository
            let media_provider_repo: Arc<dyn MediaProviderRepository> = provider_repo.clone();

            // Wrap repository with caching decorator (Decorator Pattern)
            // This makes caching transparent - business logic doesn't need manual cache checks
            let cache_repo_trait: Arc<dyn CacheRepository> = cache_repo.clone();
            let relationship_provider_repo: Arc<dyn RelationshipProviderRepository> = provider_repo.clone();
            let anime_provider_repo: Arc<dyn AnimeProviderRepository> = Arc::new(
                CachingRepositoryDecorator::new(provider_repo, cache_repo_trait)
            );

            let provider_service = Arc::new(ProviderService::new(
                anime_provider_repo,
                media_provider_repo,
                relationship_provider_repo,
            ));



            // Get database from state for service initialization
            let database = match db_state_read.get_database() {
                Ok(db) => Arc::clone(&db),
                Err(_) => {
                    log::warn!("Database unavailable during service initialization - using graceful degradation");
                    // Continue with limited functionality - services will handle unavailable database gracefully
                    return Ok(());
                }
            };

            // Initialize repositories with proper database
            // Keep concrete type for AnimeRepositoryImpl as it's needed by AnimeRelationsRepositoryImpl
            let anime_repo_impl = Arc::new(AnimeRepositoryImpl::new(Arc::clone(&database)));
            let anime_repo: Arc<dyn AnimeRepository> = anime_repo_impl.clone();
            let collection_repo: Arc<dyn CollectionRepository> = Arc::new(CollectionRepositoryImpl::new(Arc::clone(&database)));

            // Initialize anime relations repository
            let anime_relations_repo = Arc::new(
                AnimeRelationsRepositoryImpl::new(Arc::clone(&database), anime_repo_impl.clone())
            );

            // Initialize media repositories
            let anime_image_repo: Arc<dyn AnimeImageRepository> =
                Arc::new(AnimeImageRepositoryImpl::new(Arc::clone(&database)));
            let anime_video_repo: Arc<dyn AnimeVideoRepository> =
                Arc::new(AnimeVideoRepositoryImpl::new(Arc::clone(&database)));

            // Initialize core services
            let anime_service = Arc::new(AnimeService::new(
                Arc::clone(&anime_repo),
                Arc::clone(&provider_service),
            ));

            let collection_service = Arc::new(CollectionService::new(
                Arc::clone(&collection_repo),
                Arc::clone(&anime_repo),
            ));

            let import_service = Arc::new(ImportService::new(
                Arc::clone(&anime_repo),
                Arc::clone(&provider_service),
            ));

            // Initialize media services
            let media_service = Arc::new(MediaService::new(
                Arc::clone(&anime_image_repo),
                Arc::clone(&anime_video_repo),
            ));

            let media_sync_service = Arc::new(MediaSyncService::new(
                Arc::clone(&anime_image_repo),
                Arc::clone(&anime_video_repo),
                Arc::clone(&provider_service),
            ));

            // Initialize background jobs system
            let job_repository = Arc::new(JobRepositoryImpl::new(database.pool().clone()));

            // Initialize ingestion service (unified anime creation pipeline)
            let validation_service = Arc::new(ValidationService::new(
                Arc::clone(&anime_repo),
                Arc::clone(&provider_service),
            ));
            let enhancement_service = Arc::new(DataEnhancementService::new(
                Arc::clone(&provider_service),
            ));
            let ingestion_service = Arc::new(AnimeIngestionService::new(
                validation_service,
                enhancement_service,
                Arc::clone(&anime_service),
                Arc::clone(&provider_service),
                job_repository.clone(),
            ));

            // Initialize progressive relations service (new architecture)
            let relations_cache = Arc::new(RelationsCache::new());
            let anime_relations_service = Arc::new(
                AnimeRelationsService::new(
                    relations_cache,
                    Some(Arc::clone(&anime_repo)),
                    Some(Arc::clone(&anime_relations_repo)),
                    Arc::clone(&provider_service),
                    Arc::clone(&ingestion_service),
                )
            );

            // Initialize background worker
            let background_worker = Arc::new(BackgroundWorker::new(
                job_repository.clone(),
                Arc::clone(&anime_service),
                Arc::clone(&provider_service),
                Arc::clone(&anime_relations_service),
            ));

            // Start background worker using Tauri's async runtime
            // This is the proper way to start async tasks in Tauri's setup hook
            let worker = background_worker.clone();
            let worker_handle = spawn(async move {
                worker.run().await;
            });
            log::info!("Background worker initialized for anime enrichment and relations discovery");

            // Store worker handle for graceful shutdown
            app.manage(worker_handle);
            app.manage(background_worker);

            // Manage state so commands can access services via `State<T>`
            app.manage(anime_service);
            app.manage(collection_service);
            app.manage(import_service);
            app.manage(anime_relations_service);
            app.manage(provider_service);
            app.manage(media_service);
            app.manage(media_sync_service);
            app.manage(job_repository);

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Failed to run Tauri application: {}", e);
            eprintln!("Application startup failed. Please check system requirements and permissions.");
            std::process::exit(1);
        });
}
