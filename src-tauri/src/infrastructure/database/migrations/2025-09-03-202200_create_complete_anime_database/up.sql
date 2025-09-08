-- Complete anime database setup with all tables, constraints, and optimizations
-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============= CREATE TABLES =============

-- Anime table
CREATE TABLE anime (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    mal_id INTEGER NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    title_english VARCHAR(255),
    title_japanese VARCHAR(255),
    score REAL,
    scored_by INTEGER,
    rank INTEGER,
    popularity INTEGER,
    members INTEGER,
    favorites INTEGER,
    synopsis TEXT,
    episodes INTEGER,
    status VARCHAR(50) NOT NULL DEFAULT 'unknown',
    aired_from TIMESTAMPTZ,
    aired_to TIMESTAMPTZ,
    anime_type VARCHAR(50) NOT NULL DEFAULT 'Unknown',
    rating VARCHAR(50),
    source VARCHAR(100),
    duration VARCHAR(50),
    image_url TEXT,
    mal_url TEXT,
    composite_score REAL NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Genres table
CREATE TABLE genres (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    mal_id INTEGER NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL
);

-- Anime-Genre relationship
CREATE TABLE anime_genres (
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    genre_id UUID NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    PRIMARY KEY (anime_id, genre_id)
);

-- Studios table
CREATE TABLE studios (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE
);

-- Anime-Studio relationship
CREATE TABLE anime_studios (
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    studio_id UUID NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    PRIMARY KEY (anime_id, studio_id)
);

-- Collections table
CREATE TABLE collections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Collection-Anime relationship
CREATE TABLE collection_anime (
    collection_id UUID NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_score REAL CHECK (user_score >= 0 AND user_score <= 10),
    notes TEXT,
    PRIMARY KEY (collection_id, anime_id)
);

-- Quality metrics table
CREATE TABLE quality_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    popularity_score REAL NOT NULL DEFAULT 0.0,
    engagement_score REAL NOT NULL DEFAULT 0.0,
    consistency_score REAL NOT NULL DEFAULT 0.0,
    audience_reach_score REAL NOT NULL DEFAULT 0.0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(anime_id)
);

-- Collections: Add case-insensitive unique constraint for names
CREATE UNIQUE INDEX ux_collections_name_lower ON collections(LOWER(name));

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

-- ============= CREATE PERFORMANCE INDEXES =============

-- Basic indexes for anime
CREATE INDEX idx_anime_mal_id ON anime(mal_id);
CREATE INDEX idx_anime_composite_score ON anime(composite_score DESC);
CREATE INDEX idx_anime_popularity ON anime(popularity);
CREATE INDEX idx_anime_members ON anime(members DESC);
CREATE INDEX idx_anime_status ON anime(status);
CREATE INDEX idx_anime_type ON anime(anime_type);
CREATE INDEX idx_anime_aired_from ON anime(aired_from);

-- Trigram indexes for fuzzy search
CREATE INDEX idx_anime_title_trgm ON anime USING gin(LOWER(title) gin_trgm_ops);
CREATE INDEX idx_anime_title_english_trgm ON anime USING gin(LOWER(title_english) gin_trgm_ops)
    WHERE title_english IS NOT NULL;
CREATE INDEX idx_anime_title_japanese_trgm ON anime USING gin(LOWER(title_japanese) gin_trgm_ops)
    WHERE title_japanese IS NOT NULL;

-- Genre indexes
CREATE INDEX idx_genres_name ON genres(name);
CREATE INDEX idx_genres_mal_id ON genres(mal_id);

-- Junction table indexes
CREATE INDEX idx_anime_genres_anime_id ON anime_genres(anime_id);
CREATE INDEX idx_anime_genres_genre_id ON anime_genres(genre_id);

-- Studio indexes
CREATE INDEX idx_anime_studios_anime_id ON anime_studios(anime_id);
CREATE INDEX idx_anime_studios_studio_id ON anime_studios(studio_id);

-- Collection indexes
CREATE INDEX idx_collections_name ON collections(name);
CREATE INDEX idx_collections_created_at ON collections(created_at DESC);
CREATE INDEX idx_collections_name_trgm ON collections USING gin(LOWER(name) gin_trgm_ops);

CREATE INDEX idx_collection_anime_collection_id ON collection_anime(collection_id);
CREATE INDEX idx_collection_anime_anime_id ON collection_anime(anime_id);
CREATE INDEX idx_collection_anime_added_at ON collection_anime(added_at DESC);
CREATE INDEX idx_collection_anime_anime_added ON collection_anime(anime_id, added_at DESC);

-- Quality metrics indexes
CREATE INDEX idx_quality_metrics_anime_id ON quality_metrics(anime_id);

-- Composite indexes for common queries
CREATE INDEX idx_anime_status_composite_score ON anime(status, composite_score DESC);
CREATE INDEX idx_anime_type_composite_score ON anime(anime_type, composite_score DESC);

-- ============= CREATE TRIGGERS =============

-- Create triggers for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_anime_updated_at BEFORE UPDATE ON anime
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_collections_updated_at BEFORE UPDATE ON collections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_quality_metrics_updated_at BEFORE UPDATE ON quality_metrics
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add a function to clean up before anime deletion
CREATE OR REPLACE FUNCTION clean_anime_relations()
RETURNS TRIGGER AS $$
BEGIN
    -- This is handled by ON DELETE CASCADE, but we can add custom logic here if needed
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for cleanup (optional, since we have CASCADE)
CREATE TRIGGER clean_anime_relations_trigger
BEFORE DELETE ON anime
FOR EACH ROW
EXECUTE FUNCTION clean_anime_relations();

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

-- ============= CREATE MATERIALIZED VIEW FOR PERFORMANCE =============

-- Create a materialized view for genre popularity
CREATE MATERIALIZED VIEW mv_genre_popularity AS
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

-- ============= ADD COMMENTS FOR DOCUMENTATION =============

COMMENT ON TABLE anime IS 'Main anime information table';
COMMENT ON TABLE genres IS 'Genre definitions';
COMMENT ON TABLE studios IS 'Animation studio information';
COMMENT ON TABLE collections IS 'User-defined anime collections';
COMMENT ON TABLE quality_metrics IS 'Quality scoring metrics for anime';

COMMENT ON COLUMN anime.mal_id IS 'MyAnimeList ID - required and unique for all anime';
COMMENT ON COLUMN genres.mal_id IS 'MyAnimeList genre ID - required and unique for all genres';
COMMENT ON FUNCTION search_anime_fuzzy IS 'Performs fuzzy text search on anime titles using pg_trgm';
COMMENT ON FUNCTION get_collection_stats IS 'Returns aggregate statistics for a collection';
