-- Restore deprecated columns (rollback)
-- Note: This will restore columns but data will be lost

-- Drop the new materialized view
DROP MATERIALIZED VIEW IF EXISTS mv_genre_popularity;

-- Restore deprecated columns
ALTER TABLE anime
ADD COLUMN scored_by_deprecated INTEGER,
ADD COLUMN rank_deprecated INTEGER,
ADD COLUMN popularity_deprecated INTEGER,
ADD COLUMN members_deprecated INTEGER;

-- Restore the check constraints
ALTER TABLE anime ADD CONSTRAINT check_anime_members CHECK ((members_deprecated IS NULL) OR (members_deprecated >= 0));
ALTER TABLE anime ADD CONSTRAINT check_anime_scored_by CHECK ((scored_by_deprecated IS NULL) OR (scored_by_deprecated >= 0));
ALTER TABLE anime ADD CONSTRAINT check_anime_rank CHECK ((rank_deprecated IS NULL) OR (rank_deprecated > 0));
ALTER TABLE anime ADD CONSTRAINT check_anime_popularity CHECK ((popularity_deprecated IS NULL) OR (popularity_deprecated > 0));

-- Restore the original materialized view
CREATE MATERIALIZED VIEW mv_genre_popularity AS
SELECT
    g.id,
    g.name,
    count(DISTINCT ag.anime_id) AS anime_count,
    avg(a.score) AS avg_score,
    avg(a.members_deprecated) AS avg_members,
    sum(a.favorites) AS total_favorites
FROM genres g
LEFT JOIN anime_genres ag ON (g.id = ag.genre_id)
LEFT JOIN anime a ON (ag.anime_id = a.id)
GROUP BY g.id, g.name
ORDER BY count(DISTINCT ag.anime_id) DESC;
