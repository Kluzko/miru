-- ============================================================================
-- SIMPLIFY ANIME_RELATIONS TABLE
-- ============================================================================
-- Remove unnecessary columns from anime_relations table to simplify the schema.
-- Relations data will now be stored as complete anime records in the main anime table
-- instead of partial data in JSONB enrichment_data.

-- Drop indexes first (to avoid dependency issues)
DROP INDEX IF EXISTS idx_anime_relations_enrichment;
DROP INDEX IF EXISTS idx_anime_relations_enriched;

-- Remove foreign key constraint on provider_code (will be dropped)
ALTER TABLE anime_relations DROP CONSTRAINT IF EXISTS anime_relations_provider_code_fkey;

-- Remove the unnecessary columns
ALTER TABLE anime_relations DROP COLUMN IF EXISTS order_index;
ALTER TABLE anime_relations DROP COLUMN IF EXISTS enrichment_data;
ALTER TABLE anime_relations DROP COLUMN IF EXISTS enriched_at;
ALTER TABLE anime_relations DROP COLUMN IF EXISTS provider_code;
ALTER TABLE anime_relations DROP COLUMN IF EXISTS external_id;

-- Keep last_synced for tracking when relations were last discovered
-- (rename from last_synced to discovered_at for clarity)
ALTER TABLE anime_relations RENAME COLUMN last_synced TO discovered_at;
ALTER TABLE anime_relations ALTER COLUMN discovered_at SET DEFAULT CURRENT_TIMESTAMP;

-- Add comment to clarify the new approach
COMMENT ON TABLE anime_relations IS 'Simplified anime relations table. Related anime info is stored as complete records in the anime table rather than partial data.';
COMMENT ON COLUMN anime_relations.discovered_at IS 'When this relation was discovered/last updated from external providers';
