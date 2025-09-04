mod application;
mod domain;
mod infrastructure;
mod shared;

use application::{
    commands::*,
    services::{AnimeService, CollectionService, ImportService},
};
use infrastructure::{
    cache::RedisCache,
    database::{repositories::*, Database},
    external::jikan::JikanClient,
};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logging
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
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

            // Initialize cache
            let redis_url = std::env::var("REDIS_URL")?;
            let cache =
                Arc::new(RedisCache::new(&redis_url).expect("Failed to initialize Redis cache"));

            // Initialize external clients
            let jikan_client =
                Arc::new(JikanClient::new().expect("Failed to initialize Jikan client"));

            // Initialize repositories
            let anime_repo: Arc<dyn domain::repositories::AnimeRepository> =
                Arc::new(AnimeRepositoryImpl::new(Arc::clone(&database)));
            let collection_repo: Arc<dyn domain::repositories::CollectionRepository> =
                Arc::new(CollectionRepositoryImpl::new(Arc::clone(&database)));

            // Initialize services
            let anime_service = Arc::new(AnimeService::new(
                Arc::clone(&anime_repo),
                Arc::clone(&cache),
                Arc::clone(&jikan_client),
            ));

            let collection_service = Arc::new(CollectionService::new(
                Arc::clone(&collection_repo),
                Arc::clone(&anime_repo),
                Arc::clone(&cache),
            ));

            let import_service = Arc::new(ImportService::new(
                Arc::clone(&anime_repo),
                Arc::clone(&jikan_client),
            ));

            // Manage state
            app.manage(anime_service);
            app.manage(collection_service);
            app.manage(import_service);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Anime commands
            search_anime,
            get_anime_by_id,
            get_anime_by_mal_id,
            update_anime,
            delete_anime,
            get_top_anime,
            get_seasonal_anime,
            recalculate_scores,
            get_recommendations,
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
            get_collection_statistics,
            // Import commands
            import_anime_batch,
            import_from_mal_ids,
            import_from_csv,
            import_seasonal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
