-- Add unique constraint to genre name to enable proper upsert operations
-- First, remove any potential duplicate genres by keeping only the first occurrence
WITH duplicates AS (
    SELECT id, name, ROW_NUMBER() OVER (PARTITION BY name ORDER BY id) as rn
    FROM genres
)
DELETE FROM genres
WHERE id IN (
    SELECT id FROM duplicates WHERE rn > 1
);

-- Now add the unique constraint
ALTER TABLE genres ADD CONSTRAINT genres_name_unique UNIQUE (name);
