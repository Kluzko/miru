-- Revert cleanup of leftover fields
-- Note: This is mainly for development - in production, these fields shouldn't be added back

-- Add back rating column (though this shouldn't be used)
ALTER TABLE anime ADD COLUMN IF NOT EXISTS rating VARCHAR(50);

-- Add back mal_url column (though this shouldn't be used)
ALTER TABLE anime ADD COLUMN IF NOT EXISTS mal_url TEXT;

-- Remove constraints that were added
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_score_range;
ALTER TABLE anime DROP CONSTRAINT IF EXISTS check_anime_composite_score_range;
