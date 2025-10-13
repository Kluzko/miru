use tauri_specta::collect_commands;

// Import all command modules
use crate::modules::{
    anime::commands::*, collection::commands::*, data_import::commands::*, provider::commands::*,
};

/// Single source of truth for all Tauri commands
/// This eliminates the catastrophic manual synchronization requirement
/// between specta_builder and tauri::generate_handler!
pub fn get_all_commands() -> tauri_specta::Commands<tauri::Wry> {
    collect_commands![
        // Anime commands
        search_anime,
        get_anime_by_id,
        get_top_anime,
        get_seasonal_anime,
        search_anime_external,
        get_anime_by_external_id,
        get_anime_relations,
        // Auto-enrichment commands (background enrichment on loading)
        auto_enrich_on_load,
        // Relations command (single optimized call with auto-discovery)
        get_anime_with_relations,
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
        // Provider commands (AniList-exclusive franchise discovery)
        get_franchise_relations,
        discover_franchise_details,
        discover_categorized_franchise,
        get_relationship_capabilities,
    ]
}

/// Generate the handler list with all commands imported
#[macro_export]
macro_rules! generate_handler_list {
    () => {{
        use crate::modules::{
            anime::commands::*, collection::commands::*, data_import::commands::*,
            provider::commands::*,
        };

        tauri::generate_handler![
            // Anime commands
            search_anime,
            get_anime_by_id,
            get_top_anime,
            get_seasonal_anime,
            search_anime_external,
            get_anime_by_external_id,
            get_anime_relations,
            // Auto-enrichment commands (background enrichment on loading)
            auto_enrich_on_load,
            // Progressive relations commands (simplified to single command)
            get_anime_with_relations,
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
            // Provider commands (AniList-exclusive franchise discovery)
            get_franchise_relations,
            discover_franchise_details,
            discover_categorized_franchise,
            get_relationship_capabilities,
        ]
    }};
}
