-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS update_anime_updated_at ON anime;
DROP TRIGGER IF EXISTS update_collections_updated_at ON collections;
DROP TRIGGER IF EXISTS update_quality_metrics_updated_at ON quality_metrics;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop indexes
DROP INDEX IF EXISTS idx_anime_mal_id;
DROP INDEX IF EXISTS idx_anime_title;
DROP INDEX IF EXISTS idx_anime_title_english;
DROP INDEX IF EXISTS idx_anime_title_japanese;
DROP INDEX IF EXISTS idx_anime_composite_score;
DROP INDEX IF EXISTS idx_anime_popularity;
DROP INDEX IF EXISTS idx_anime_members;
DROP INDEX IF EXISTS idx_anime_status;
DROP INDEX IF EXISTS idx_anime_type;
DROP INDEX IF EXISTS idx_anime_aired_from;

DROP INDEX IF EXISTS idx_genres_name;
DROP INDEX IF EXISTS idx_genres_mal_id;

DROP INDEX IF EXISTS idx_anime_genres_anime_id;
DROP INDEX IF EXISTS idx_anime_genres_genre_id;

DROP INDEX IF EXISTS idx_studios_name;

DROP INDEX IF EXISTS idx_anime_studios_anime_id;
DROP INDEX IF EXISTS idx_anime_studios_studio_id;

DROP INDEX IF EXISTS idx_collections_name;
DROP INDEX IF EXISTS idx_collections_created_at;

DROP INDEX IF EXISTS idx_collection_anime_collection_id;
DROP INDEX IF EXISTS idx_collection_anime_anime_id;
DROP INDEX IF EXISTS idx_collection_anime_added_at;

DROP INDEX IF EXISTS idx_quality_metrics_anime_id;

-- Drop tables in correct order
DROP TABLE IF EXISTS quality_metrics;
DROP TABLE IF EXISTS collection_anime;
DROP TABLE IF EXISTS collections;
DROP TABLE IF EXISTS anime_studios;
DROP TABLE IF EXISTS studios;
DROP TABLE IF EXISTS anime_genres;
DROP TABLE IF EXISTS genres;
DROP TABLE IF EXISTS anime;
