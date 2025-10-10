pub mod commands;
pub mod modules;
mod schema;
pub mod shared;

use commands::get_all_commands;
use modules::{
    anime::{
        application::service::AnimeService,
        domain::services::anime_relations_service::{AnimeRelationsService, RelationsCache},
        infrastructure::persistence::AnimeRepositoryImpl,
        AnimeRepository,
    },
    collection::{
        application::service::CollectionService,
        infrastructure::persistence::CollectionRepositoryImpl, CollectionRepository,
    },
    data_import::application::service::ImportService,
    provider::{
        application::service::ProviderService,
        infrastructure::adapters::{CacheAdapter, ProviderRepositoryAdapter},
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

    // Initialize structured logging
    shared::utils::logger::init_logger();

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
            let provider_service = Arc::new(ProviderService::new(provider_repo, cache_repo));



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
            let anime_repo: Arc<dyn AnimeRepository> = Arc::new(AnimeRepositoryImpl::new(Arc::clone(&database)));
            let collection_repo: Arc<dyn CollectionRepository> = Arc::new(CollectionRepositoryImpl::new(Arc::clone(&database)));

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

            // Initialize progressive relations service (new architecture)
            let relations_cache = Arc::new(RelationsCache::new());
            let anime_relations_service = Arc::new(AnimeRelationsService::new(
                relations_cache,
                Some(Arc::clone(&anime_repo)),
                Arc::clone(&provider_service),
            ));

            // Manage state so commands can access services via `State<T>`
            app.manage(anime_service);
            app.manage(collection_service);
            app.manage(import_service);
            app.manage(anime_relations_service);
            app.manage(provider_service);

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Failed to run Tauri application: {}", e);
            eprintln!("Application startup failed. Please check system requirements and permissions.");
            std::process::exit(1);
        });
}
