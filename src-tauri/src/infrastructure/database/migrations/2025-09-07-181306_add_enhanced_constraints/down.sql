
-- Drop materialized views and functions
DROP MATERIALIZED VIEW IF EXISTS mv_genre_popularity;
DROP FUNCTION IF EXISTS refresh_materialized_views();
DROP FUNCTION IF EXISTS get_collection_stats(UUID);
DROP FUNCTION IF EXISTS search_anime_fuzzy(TEXT, REAL, INTEGER);

-- Drop trigram indexes
DROP INDEX IF EXISTS idx_anime_title_trgm;
DROP INDEX IF EXISTS idx_anime_title_english_trgm;
DROP INDEX IF EXISTS idx_anime_title_japanese_trgm;
DROP INDEX IF EXISTS idx_collections_name_trgm;

-- Drop composite indexes
DROP INDEX IF EXISTS idx_anime_status_composite_score;
DROP INDEX IF EXISTS idx_anime_type_composite_score;
DROP INDEX IF EXISTS idx_collection_anime_anime_added;

-- Drop check constraints from anime table
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_score;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_composite_score;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_members;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_favorites;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_episodes;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_scored_by;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_rank;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_popularity;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_aired_dates;

-- Drop check constraints from quality_metrics table
ALTER TABLE quality_metrics DROP CONSTRAINT IF EXISTS check_quality_metrics_scores;

-- Drop unique indexes (reverting to non-unique)
DROP INDEX IF EXISTS ux_genres_mal_id;
DROP INDEX IF EXISTS ux_genres_name_lower;
DROP INDEX IF EXISTS ux_studios_name_lower;
DROP INDEX IF EXISTS ux_collections_name_lower;

-- Recreate original unique constraints (without case-insensitive)
ALTER TABLE genres ADD CONSTRAINT genres_mal_id_key UNIQUE(mal_id);
ALTER TABLE genres ADD CONSTRAINT genres_name_key UNIQUE(name);
ALTER TABLE studios ADD CONSTRAINT studios_name_key UNIQUE(name);

-- Recreate basic indexes
CREATE INDEX idx_anime_title ON anime(title);
CREATE INDEX idx_anime_title_english ON anime(title_english);
CREATE INDEX idx_anime_title_japanese ON anime(title_japanese);

-- Note: We don't drop pg_trgm extension as other parts of the database might be using it
