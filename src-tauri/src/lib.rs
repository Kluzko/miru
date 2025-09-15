mod application;
mod domain;
mod infrastructure;
mod shared;

// Log macros are exported by the logger module

use application::{
    commands::*,
    services::{AnimeService, CollectionService, ImportService, ProviderManager},
};
use infrastructure::database::{repositories::*, Database};
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
        import_from_csv,
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
    specta_builder
        .export(Typescript::default(), "../src/types/bindings.ts")
        .expect("tauri-specta: failed to export TypeScript bindings");

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

            // Initialize database
            let database =
                Arc::new(Database::new().expect("Failed to initialize database connection"));

            // Run migrations
            {
                use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
                const MIGRATIONS: EmbeddedMigrations =
                    embed_migrations!("src/infrastructure/database/migrations");

                let mut conn = database
                    .get_connection()
                    .expect("Failed to get database connection for migrations");

                conn.run_pending_migrations(MIGRATIONS)
                    .expect("Failed to run database migrations");
            }

            // Initialize provider manager
            let provider_manager = Arc::new(tokio::sync::Mutex::new(ProviderManager::new()));

            // Initialize repositories
            let anime_repo: Arc<dyn domain::repositories::AnimeRepository> =
                Arc::new(AnimeRepositoryImpl::new(Arc::clone(&database)));
            let collection_repo: Arc<dyn domain::repositories::CollectionRepository> =
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
        .expect("error while running tauri application");
}
