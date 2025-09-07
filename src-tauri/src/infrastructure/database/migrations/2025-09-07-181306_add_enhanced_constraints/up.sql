-- Your SQL goes here

-- Enable pg_trgm extension for fuzzy text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============= FIX UNIQUE CONSTRAINTS (P0 #3) =============

-- Drop old non-unique indexes that should be unique
DROP INDEX IF EXISTS idx_genres_mal_id_unique;
DROP INDEX IF EXISTS idx_genres_name_unique;
DROP INDEX IF EXISTS idx_studios_name_unique;

-- Genres: Fix partial unique constraints
-- First, clean up any remaining duplicates
DELETE FROM genres g1
WHERE g1.mal_id IS NOT NULL
AND EXISTS (
    SELECT 1 FROM genres g2
    WHERE g2.mal_id = g1.mal_id AND g2.id < g1.id
);

DELETE FROM genres g1
WHERE g1.mal_id IS NULL
AND EXISTS (
    SELECT 1 FROM genres g2
    WHERE LOWER(g2.name) = LOWER(g1.name) AND g2.mal_id IS NULL AND g2.id < g1.id
);

-- Now add proper unique constraints
ALTER TABLE genres DROP CONSTRAINT IF EXISTS genres_mal_id_key;
ALTER TABLE genres DROP CONSTRAINT IF EXISTS genres_name_key;

-- Partial unique index for mal_id (when not null)
CREATE UNIQUE INDEX ux_genres_mal_id ON genres(mal_id) WHERE mal_id IS NOT NULL;

-- Partial unique index for name (case-insensitive, when mal_id is null)
CREATE UNIQUE INDEX ux_genres_name_lower ON genres(LOWER(name)) WHERE mal_id IS NULL;

-- Studios: Fix case-insensitive unique constraint
-- First, clean up duplicates
DELETE FROM studios s1
WHERE EXISTS (
    SELECT 1 FROM studios s2
    WHERE LOWER(s2.name) = LOWER(s1.name) AND s2.id < s1.id
);

ALTER TABLE studios DROP CONSTRAINT IF EXISTS studios_name_key;
CREATE UNIQUE INDEX ux_studios_name_lower ON studios(LOWER(name));

-- Collections: Add case-insensitive unique constraint for names
DELETE FROM collections c1
WHERE EXISTS (
    SELECT 1 FROM collections c2
    WHERE LOWER(c2.name) = LOWER(c1.name) AND c2.id < c1.id
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_collections_name_lower ON collections(LOWER(name));

-- ============= ADD CHECK CONSTRAINTS =============

-- Anime table: Add data validation constraints
ALTER TABLE anime ADD CONSTRAINT check_anime_score
    CHECK (score IS NULL OR (score >= 0 AND score <= 10));

ALTER TABLE anime ADD CONSTRAINT check_anime_composite_score
    CHECK (composite_score >= 0 AND composite_score <= 10);

ALTER TABLE anime ADD CONSTRAINT check_anime_members
    CHECK (members IS NULL OR members >= 0);

ALTER TABLE anime ADD CONSTRAINT check_anime_favorites
    CHECK (favorites IS NULL OR favorites >= 0);

ALTER TABLE anime ADD CONSTRAINT check_anime_episodes
    CHECK (episodes IS NULL OR episodes >= 0);

ALTER TABLE anime ADD CONSTRAINT check_anime_scored_by
    CHECK (scored_by IS NULL OR scored_by >= 0);

ALTER TABLE anime ADD CONSTRAINT check_anime_rank
    CHECK (rank IS NULL OR rank > 0);

ALTER TABLE anime ADD CONSTRAINT check_anime_popularity
    CHECK (popularity IS NULL OR popularity > 0);

ALTER TABLE anime ADD CONSTRAINT check_anime_aired_dates
    CHECK (aired_from IS NULL OR aired_to IS NULL OR aired_to >= aired_from);

-- Quality metrics: Add validation constraints
ALTER TABLE quality_metrics ADD CONSTRAINT check_quality_metrics_scores
    CHECK (
        popularity_score >= 0 AND popularity_score <= 10 AND
        engagement_score >= 0 AND engagement_score <= 10 AND
        consistency_score >= 0 AND consistency_score <= 10 AND
        audience_reach_score >= 0 AND audience_reach_score <= 10
    );

-- ============= ADD TRIGRAM INDEXES FOR SEARCH (P0 #7) =============

-- Drop old basic indexes
DROP INDEX IF EXISTS idx_anime_title;
DROP INDEX IF EXISTS idx_anime_title_english;
DROP INDEX IF EXISTS idx_anime_title_japanese;

-- Create GIN trigram indexes for fuzzy search
CREATE INDEX idx_anime_title_trgm ON anime USING gin(LOWER(title) gin_trgm_ops);
CREATE INDEX idx_anime_title_english_trgm ON anime USING gin(LOWER(title_english) gin_trgm_ops)
    WHERE title_english IS NOT NULL;
CREATE INDEX idx_anime_title_japanese_trgm ON anime USING gin(LOWER(title_japanese) gin_trgm_ops)
    WHERE title_japanese IS NOT NULL;

-- Add trigram index for collection names too
CREATE INDEX idx_collections_name_trgm ON collections USING gin(LOWER(name) gin_trgm_ops);

-- ============= OPTIMIZE EXISTING INDEXES =============

-- Add better composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_anime_status_composite_score
    ON anime(status, composite_score DESC);

CREATE INDEX IF NOT EXISTS idx_anime_type_composite_score
    ON anime(anime_type, composite_score DESC);

-- Optimize collection_anime queries
CREATE INDEX IF NOT EXISTS idx_collection_anime_anime_added
    ON collection_anime(anime_id, added_at DESC);

-- ============= CREATE HELPER FUNCTIONS =============

-- Function for fuzzy anime search using pg_trgm
CREATE OR REPLACE FUNCTION search_anime_fuzzy(
    search_query TEXT,
    similarity_threshold REAL DEFAULT 0.3,
    max_results INTEGER DEFAULT 20
)
RETURNS TABLE(
    id UUID,
    title VARCHAR(255),
    similarity_score REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        a.id,
        a.title,
        GREATEST(
            similarity(LOWER(a.title), LOWER(search_query)),
            COALESCE(similarity(LOWER(a.title_english), LOWER(search_query)), 0),
            COALESCE(similarity(LOWER(a.title_japanese), LOWER(search_query)), 0)
        ) AS similarity_score
    FROM anime a
    WHERE
        similarity(LOWER(a.title), LOWER(search_query)) > similarity_threshold OR
        similarity(LOWER(a.title_english), LOWER(search_query)) > similarity_threshold OR
        similarity(LOWER(a.title_japanese), LOWER(search_query)) > similarity_threshold
    ORDER BY similarity_score DESC, a.composite_score DESC
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- Function to get collection statistics
CREATE OR REPLACE FUNCTION get_collection_stats(collection_uuid UUID)
RETURNS TABLE(
    total_anime BIGINT,
    total_episodes BIGINT,
    avg_score NUMERIC,
    avg_user_score NUMERIC,
    total_members BIGINT,
    total_favorites BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        COUNT(DISTINCT ca.anime_id) AS total_anime,
        SUM(a.episodes) AS total_episodes,
        ROUND(AVG(a.score)::NUMERIC, 2) AS avg_score,
        ROUND(AVG(ca.user_score)::NUMERIC, 2) AS avg_user_score,
        SUM(a.members) AS total_members,
        SUM(a.favorites) AS total_favorites
    FROM collection_anime ca
    JOIN anime a ON ca.anime_id = a.id
    WHERE ca.collection_id = collection_uuid;
END;
$$ LANGUAGE plpgsql;

-- ============= ADD COMMENTS FOR DOCUMENTATION =============

COMMENT ON INDEX ux_genres_mal_id IS 'Unique constraint for genres with MAL ID';
COMMENT ON INDEX ux_genres_name_lower IS 'Case-insensitive unique constraint for genres without MAL ID';
COMMENT ON INDEX ux_studios_name_lower IS 'Case-insensitive unique constraint for studio names';
COMMENT ON INDEX ux_collections_name_lower IS 'Case-insensitive unique constraint for collection names';
COMMENT ON FUNCTION search_anime_fuzzy IS 'Performs fuzzy text search on anime titles using pg_trgm';
COMMENT ON FUNCTION get_collection_stats IS 'Returns aggregate statistics for a collection';

-- ============= CREATE MATERIALIZED VIEW FOR PERFORMANCE =============

-- Create a materialized view for genre popularity
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_genre_popularity AS
SELECT
    g.id,
    g.name,
    COUNT(DISTINCT ag.anime_id) as anime_count,
    AVG(a.score) as avg_score,
    AVG(a.members) as avg_members,
    SUM(a.favorites) as total_favorites
FROM genres g
LEFT JOIN anime_genres ag ON g.id = ag.genre_id
LEFT JOIN anime a ON ag.anime_id = a.id
GROUP BY g.id, g.name
ORDER BY anime_count DESC;

CREATE UNIQUE INDEX idx_mv_genre_popularity_id ON mv_genre_popularity(id);
CREATE INDEX idx_mv_genre_popularity_count ON mv_genre_popularity(anime_count DESC);

-- Function to refresh materialized views
CREATE OR REPLACE FUNCTION refresh_materialized_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_genre_popularity;
END;
$$ LANGUAGE plpgsql;

-- ============= END OF UP MIGRATION =============
