-- Drop deprecated columns that are no longer used by the application
-- These were renamed in the previous migration to ensure safe transition

-- First, drop the materialized view that depends on deprecated columns
DROP MATERIALIZED VIEW IF EXISTS mv_genre_popularity;

-- Drop the check constraints that reference these columns
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_members;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_scored_by;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_rank;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_popularity;

-- Now drop the deprecated columns
ALTER TABLE anime
DROP COLUMN IF EXISTS scored_by_deprecated,
DROP COLUMN IF EXISTS rank_deprecated,
DROP COLUMN IF EXISTS popularity_deprecated,
DROP COLUMN IF EXISTS members_deprecated;

-- Recreate the materialized view without deprecated columns
-- Use favorites as engagement metric instead of members_deprecated
CREATE MATERIALIZED VIEW mv_genre_popularity AS
SELECT
    g.id,
    g.name,
    count(DISTINCT ag.anime_id) AS anime_count,
    avg(a.score) AS avg_score,
    avg(a.favorites) AS avg_favorites,
    sum(a.favorites) AS total_favorites
FROM genres g
LEFT JOIN anime_genres ag ON (g.id = ag.genre_id)
LEFT JOIN anime a ON (ag.anime_id = a.id)
GROUP BY g.id, g.name
ORDER BY count(DISTINCT ag.anime_id) DESC;
