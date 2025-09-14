-- Rollback: Restore original anime scoring fields

-- Remove unified field
ALTER TABLE anime DROP COLUMN IF EXISTS last_synced_at;

-- Restore original column names
ALTER TABLE anime RENAME COLUMN scored_by_deprecated TO scored_by;
ALTER TABLE anime RENAME COLUMN rank_deprecated TO rank;
ALTER TABLE anime RENAME COLUMN popularity_deprecated TO popularity;
ALTER TABLE anime RENAME COLUMN members_deprecated TO members;
