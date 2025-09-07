-- This file should undo anything in `up.sql`

-- Remove the trigger and function
DROP TRIGGER IF EXISTS clean_anime_relations_trigger ON anime;
DROP FUNCTION IF EXISTS clean_anime_relations();

-- Remove the indexes
DROP INDEX IF EXISTS idx_anime_mal_id_unique;
DROP INDEX IF EXISTS idx_genres_mal_id_unique;
DROP INDEX IF EXISTS idx_genres_name_unique;
DROP INDEX IF EXISTS idx_studios_name_unique;
