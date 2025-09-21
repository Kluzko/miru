-- Remove leftover fields from previous migrations
-- These fields are no longer used in the new unified schema

-- Remove rating column (replaced by age_restriction)
ALTER TABLE anime DROP COLUMN IF EXISTS rating;

-- Remove mal_url column (replaced by anime_external_ids table)
ALTER TABLE anime DROP COLUMN IF EXISTS mal_url;

-- Make sure all NOT NULL constraints are properly set
ALTER TABLE anime ALTER COLUMN title_main SET NOT NULL;
ALTER TABLE anime ALTER COLUMN status SET NOT NULL;
ALTER TABLE anime ALTER COLUMN anime_type SET NOT NULL;
ALTER TABLE anime ALTER COLUMN tier SET NOT NULL;

-- Update schema to ensure consistency
-- Ensure composite_score has a proper default
ALTER TABLE anime ALTER COLUMN composite_score SET DEFAULT 0.0;

-- Add any missing constraints that should be present
ALTER TABLE anime ADD CONSTRAINT check_anime_score_range
    CHECK (score IS NULL OR (score >= 0 AND score <= 10));

ALTER TABLE anime ADD CONSTRAINT check_anime_composite_score_range
    CHECK (composite_score >= 0 AND composite_score <= 10);

-- Add comment for documentation
COMMENT ON TABLE anime IS 'Main anime table with unified schema - cleaned up from legacy fields';
