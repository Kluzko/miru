-- Drop complete anime database
-- Your SQL goes here

-- Drop materialized view and functions
DROP MATERIALIZED VIEW IF EXISTS mv_genre_popularity CASCADE;
DROP FUNCTION IF EXISTS refresh_materialized_views();
DROP FUNCTION IF EXISTS search_anime_fuzzy(TEXT, REAL, INTEGER);
DROP FUNCTION IF EXISTS get_collection_stats(UUID);
DROP FUNCTION IF EXISTS clean_anime_relations();
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop triggers
DROP TRIGGER IF EXISTS clean_anime_relations_trigger ON anime;
DROP TRIGGER IF EXISTS update_anime_updated_at ON anime;
DROP TRIGGER IF EXISTS update_collections_updated_at ON collections;
DROP TRIGGER IF EXISTS update_quality_metrics_updated_at ON quality_metrics;

-- Drop indexes
DROP INDEX IF EXISTS idx_anime_mal_id_unique;
DROP INDEX IF EXISTS idx_genres_mal_id_unique;
DROP INDEX IF EXISTS idx_genres_name_unique_when_no_mal;
DROP INDEX IF EXISTS idx_anime_title_trgm;
DROP INDEX IF EXISTS idx_anime_title_english_trgm;
DROP INDEX IF EXISTS idx_anime_title_japanese_trgm;
DROP INDEX IF EXISTS idx_collections_name_trgm;
DROP INDEX IF EXISTS idx_anime_status_composite_score;
DROP INDEX IF EXISTS idx_anime_type_composite_score;
DROP INDEX IF EXISTS idx_collection_anime_anime_added;
DROP INDEX IF EXISTS ux_collections_name_lower;
DROP INDEX IF EXISTS idx_mv_genre_popularity_id;
DROP INDEX IF EXISTS idx_mv_genre_popularity_count;

-- Drop all other indexes
DROP INDEX IF EXISTS idx_anime_composite_score;
DROP INDEX IF EXISTS idx_anime_popularity;
DROP INDEX IF EXISTS idx_anime_members;
DROP INDEX IF EXISTS idx_anime_status;
DROP INDEX IF EXISTS idx_anime_type;
DROP INDEX IF EXISTS idx_anime_aired_from;
DROP INDEX IF EXISTS idx_genres_name;
DROP INDEX IF EXISTS idx_anime_genres_anime_id;
DROP INDEX IF EXISTS idx_anime_genres_genre_id;
DROP INDEX IF EXISTS idx_anime_studios_anime_id;
DROP INDEX IF EXISTS idx_anime_studios_studio_id;
DROP INDEX IF EXISTS idx_collections_name;
DROP INDEX IF EXISTS idx_collections_created_at;
DROP INDEX IF EXISTS idx_collection_anime_collection_id;
DROP INDEX IF EXISTS idx_collection_anime_anime_id;
DROP INDEX IF EXISTS idx_collection_anime_added_at;
DROP INDEX IF EXISTS idx_quality_metrics_anime_id;

-- Drop tables (in dependency order)
DROP TABLE IF EXISTS quality_metrics CASCADE;
DROP TABLE IF EXISTS collection_anime CASCADE;
DROP TABLE IF EXISTS collections CASCADE;
DROP TABLE IF EXISTS anime_studios CASCADE;
DROP TABLE IF EXISTS studios CASCADE;
DROP TABLE IF EXISTS anime_genres CASCADE;
DROP TABLE IF EXISTS genres CASCADE;
DROP TABLE IF EXISTS anime CASCADE;

-- Drop extensions (only if no other objects depend on them)
-- DROP EXTENSION IF EXISTS pg_trgm;
-- DROP EXTENSION IF EXISTS "uuid-ossp";
