-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS idx_collection_anime_unique;
DROP INDEX IF EXISTS idx_anime_studios_unique;
DROP INDEX IF EXISTS idx_anime_genres_unique;
DROP INDEX IF EXISTS idx_quality_metrics_anime_id_unique;
DROP INDEX IF EXISTS idx_studios_name_unique;
DROP INDEX IF EXISTS idx_genres_name_unique_when_no_mal;
DROP INDEX IF EXISTS idx_genres_mal_id_unique;
DROP INDEX IF EXISTS idx_anime_mal_id_unique;
-- extension can be left installed; dropping it may break other objects
