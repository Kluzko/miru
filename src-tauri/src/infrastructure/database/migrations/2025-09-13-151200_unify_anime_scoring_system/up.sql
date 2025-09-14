-- Migration: Unify anime scoring system
-- Remove external API dependencies and add unified fields

-- First, add the new column before dropping old ones
ALTER TABLE anime ADD COLUMN IF NOT EXISTS last_synced_at TIMESTAMPTZ;

-- Drop any views or dependent objects that might reference the columns we want to drop
-- (We'll need to check what depends on these columns)

-- For safety, let's rename columns instead of dropping them initially
-- This preserves data while allowing the code to work with the new structure
ALTER TABLE anime RENAME COLUMN scored_by TO scored_by_deprecated;
ALTER TABLE anime RENAME COLUMN rank TO rank_deprecated;
ALTER TABLE anime RENAME COLUMN popularity TO popularity_deprecated;
ALTER TABLE anime RENAME COLUMN members TO members_deprecated;

-- Add comments to explain unified system
COMMENT ON COLUMN anime.score IS 'Unified score (0-10 scale) from all providers';
COMMENT ON COLUMN anime.favorites IS 'Primary engagement metric from providers';
COMMENT ON COLUMN anime.composite_score IS 'Internal calculated score independent of external APIs';
COMMENT ON COLUMN anime.last_synced_at IS 'Last time data was synced from external providers';
