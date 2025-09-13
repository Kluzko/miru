-- Remove mal_id field since we now use anime_external_ids table for provider metadata

-- First, migrate existing mal_id data to anime_external_ids table
INSERT INTO anime_external_ids (anime_id, provider_code, external_id, is_primary)
SELECT
    id,
    'jikan'::VARCHAR(20),
    mal_id::VARCHAR(255),
    true
FROM anime
WHERE mal_id > 0
ON CONFLICT (anime_id, provider_code) DO NOTHING;

-- Drop the unique constraint on mal_id first
ALTER TABLE anime DROP CONSTRAINT IF EXISTS anime_mal_id_key;

-- Remove the mal_id field
ALTER TABLE anime DROP COLUMN IF EXISTS mal_id;

-- Update genres table - remove mal_id constraint and make it nullable
ALTER TABLE genres DROP CONSTRAINT IF EXISTS genres_mal_id_key;
ALTER TABLE genres ALTER COLUMN mal_id DROP NOT NULL;

-- Add comment for documentation
COMMENT ON TABLE anime IS 'Main anime table - now uses anime_external_ids for provider mapping';
COMMENT ON TABLE anime_external_ids IS 'External provider ID mappings - replaces direct mal_id references';
