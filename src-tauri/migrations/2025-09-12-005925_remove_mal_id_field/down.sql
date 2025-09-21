-- Revert removal of mal_id field
-- Note: This is mainly for development rollback

-- Add back mal_id column to anime table
ALTER TABLE anime ADD COLUMN mal_id INTEGER;

-- Populate mal_id from anime_external_ids for jikan provider
UPDATE anime
SET mal_id = CAST(aei.external_id AS INTEGER)
FROM anime_external_ids aei
WHERE anime.id = aei.anime_id
AND aei.provider_code = 'jikan'
AND aei.external_id ~ '^[0-9]+$';

-- Set default value for any remaining NULL mal_ids
UPDATE anime SET mal_id = 0 WHERE mal_id IS NULL;

-- Make mal_id NOT NULL and add unique constraint
ALTER TABLE anime ALTER COLUMN mal_id SET NOT NULL;
ALTER TABLE anime ADD CONSTRAINT anime_mal_id_key UNIQUE (mal_id);

-- Restore genres mal_id constraint
ALTER TABLE genres ALTER COLUMN mal_id SET NOT NULL;
ALTER TABLE genres ADD CONSTRAINT genres_mal_id_key UNIQUE (mal_id);
