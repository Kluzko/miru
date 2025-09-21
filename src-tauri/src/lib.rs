mod modules;
mod schema;
mod shared;

// Log macros are exported by the logger module

use modules::{
    anime::{commands::*, infrastructure::persistence::AnimeRepositoryImpl, AnimeService},
    collection::{
        commands::*, infrastructure::persistence::CollectionRepositoryImpl, CollectionService,
    },
    data_import::{commands::*, ImportService},
    provider::{commands::*, ProviderManager},
};
use shared::database::Database;
use shared::validation::{
    validation_chain::ValidationChain,
    validation_rules::{ExternalIdValidationRule, ScoreValidationRule, TitleValidationRule},
};
// use shared::utils::logger::{LogContext, TimedOperation};
use std::sync::Arc;
use tauri::Manager;

// tauri-specta: generate TS types + typed command client from Rust commands
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder as SpectaBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize structured logging
    shared::utils::logger::init_logger();

    // 1) Build the specta builder with all commands
    let specta_builder = SpectaBuilder::<tauri::Wry>::new().commands(collect_commands![
        // Anime commands
        search_anime,
        get_anime_by_id,
        get_top_anime,
        get_seasonal_anime,
        // Collection commands
        create_collection,
        get_collection,
        get_all_collections,
        update_collection,
        delete_collection,
        add_anime_to_collection,
        remove_anime_from_collection,
        get_collection_anime,
        update_anime_in_collection,
        // Import commands
        import_anime_batch,
        validate_anime_titles,
        import_validated_anime,
        // Provider commands
        list_providers,
        set_primary_provider,
        get_primary_provider,
        get_enabled_providers,
        get_provider_rate_limit,
        get_age_restrictions,
    ]);

    // 2) Export bindings in debug builds
    #[cfg(debug_assertions)]
    if let Err(e) = specta_builder.export(Typescript::default(), "../src/types/bindings.ts") {
        eprintln!("Warning: Failed to export TypeScript bindings: {}", e);
        eprintln!("TypeScript types may be out of sync. Consider running cargo build again.");
    }

    // 3) Create the invoke handler BEFORE moving `specta_builder` into the setup closure
    let invoke_handler = specta_builder.invoke_handler();

    tauri::Builder::default()
        // Tell Tauri how to invoke commands (uses the handler we created above)
        .invoke_handler(invoke_handler)
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // If you want typed events, mount specta's event hooks here.
            // `specta_builder` is moved into this closure (no later uses outside).
            specta_builder.mount_events(app);

            // Initialize database with proper error handling
            let database = match Database::new() {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    eprintln!("Failed to initialize database connection: {}", e);
                    eprintln!("Please check your DATABASE_URL environment variable and database connection.");
                    std::process::exit(1);
                }
            };

            // Run migrations with proper error handling
            {
                use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
                const MIGRATIONS: EmbeddedMigrations =
                    embed_migrations!("migrations");

                let mut conn = match database.get_connection() {
                    Ok(conn) => conn,
                    Err(e) => {
                        eprintln!("Failed to get database connection for migrations: {}", e);
                        eprintln!("Database may be unreachable or configuration is incorrect.");
                        std::process::exit(1);
                    }
                };

                if let Err(e) = conn.run_pending_migrations(MIGRATIONS) {
                    eprintln!("Failed to run database migrations: {}", e);
                    eprintln!("Database migration failed. Please check database schema and permissions.");
                    std::process::exit(1);
                }
            }

            // Initialize provider manager
            let provider_manager = Arc::new(tokio::sync::Mutex::new(ProviderManager::new()));

            // Initialize repositories
            let anime_repo: Arc<dyn modules::anime::AnimeRepository> =
                Arc::new(AnimeRepositoryImpl::new(Arc::clone(&database)));
            let collection_repo: Arc<dyn modules::collection::CollectionRepository> =
                Arc::new(CollectionRepositoryImpl::new(Arc::clone(&database)));

            // Initialize services
            let anime_service = Arc::new(AnimeService::new(
                Arc::clone(&anime_repo),
                Arc::clone(&provider_manager),
            ));

            let collection_service = Arc::new(CollectionService::new(
                Arc::clone(&collection_repo),
                Arc::clone(&anime_repo),
            ));

            // Create validation chain with rules
            let _validation_chain = ValidationChain::new()
                .add_rule(Arc::new(TitleValidationRule))
                .add_rule(Arc::new(ScoreValidationRule))
                .add_rule(Arc::new(ExternalIdValidationRule));

            let import_service = Arc::new(ImportService::new(
                Arc::clone(&anime_repo),
                Arc::clone(&provider_manager),
            ));

            // Manage state so commands can access services via `State<T>`
            app.manage(anime_service);
            app.manage(collection_service);
            app.manage(import_service);
            app.manage(provider_manager);

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Failed to run Tauri application: {}", e);
            eprintln!("Application startup failed. Please check system requirements and permissions.");
            std::process::exit(1);
        });
}
