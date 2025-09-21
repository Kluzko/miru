-- Remove search performance indexes

DROP INDEX IF EXISTS idx_anime_title_search;
DROP INDEX IF EXISTS idx_anime_synonyms_gin;
DROP INDEX IF EXISTS idx_anime_composite_score_desc;
DROP INDEX IF EXISTS idx_anime_external_ids_lookup;
DROP INDEX IF EXISTS idx_anime_external_ids_anime;
DROP INDEX IF EXISTS idx_collection_anime_lookup;
DROP INDEX IF EXISTS idx_anime_genres_anime;
DROP INDEX IF EXISTS idx_anime_studios_anime;
DROP INDEX IF EXISTS idx_anime_score_notnull;
DROP INDEX IF EXISTS idx_anime_status;
DROP INDEX IF EXISTS idx_anime_type;
DROP INDEX IF EXISTS idx_anime_aired_from_date;
