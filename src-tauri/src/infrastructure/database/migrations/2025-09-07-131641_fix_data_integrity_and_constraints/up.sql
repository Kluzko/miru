-- Your SQL goes here

-- 1. Clean up orphaned anime_genres entries
DELETE FROM anime_genres
WHERE anime_id NOT IN (SELECT id FROM anime);

-- 2. Clean up orphaned anime_studios entries
DELETE FROM anime_studios
WHERE anime_id NOT IN (SELECT id FROM anime);

-- 3. Clean up orphaned quality_metrics entries
DELETE FROM quality_metrics
WHERE anime_id NOT IN (SELECT id FROM anime);

-- 4. Clean up orphaned collection_anime entries
DELETE FROM collection_anime
WHERE anime_id NOT IN (SELECT id FROM anime);

-- 5. Remove duplicate genres by mal_id (keep the first one)
DELETE FROM genres g1
WHERE g1.mal_id IS NOT NULL
AND EXISTS (
    SELECT 1
    FROM genres g2
    WHERE g2.mal_id = g1.mal_id
    AND g2.id < g1.id
);

-- 6. Remove duplicate genres by name (for those without mal_id)
DELETE FROM genres g1
WHERE g1.mal_id IS NULL
AND EXISTS (
    SELECT 1
    FROM genres g2
    WHERE g2.name = g1.name
    AND g2.mal_id IS NULL
    AND g2.id < g1.id
);

-- 7. Remove duplicate studios by name
DELETE FROM studios s1
WHERE EXISTS (
    SELECT 1
    FROM studios s2
    WHERE s2.name = s1.name
    AND s2.id < s1.id
);

-- 8. Add missing indexes for better performance
CREATE INDEX IF NOT EXISTS idx_anime_mal_id_unique ON anime(mal_id) WHERE mal_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_genres_mal_id_unique ON genres(mal_id) WHERE mal_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_genres_name_unique ON genres(name) WHERE mal_id IS NULL;
CREATE INDEX IF NOT EXISTS idx_studios_name_unique ON studios(name);

-- 9. Add a function to clean up before anime deletion
CREATE OR REPLACE FUNCTION clean_anime_relations()
RETURNS TRIGGER AS $$
BEGIN
    -- This is handled by ON DELETE CASCADE, but we can add custom logic here if needed
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- 10. Create trigger for cleanup (optional, since we have CASCADE)
DROP TRIGGER IF EXISTS clean_anime_relations_trigger ON anime;
CREATE TRIGGER clean_anime_relations_trigger
BEFORE DELETE ON anime
FOR EACH ROW
EXECUTE FUNCTION clean_anime_relations();
