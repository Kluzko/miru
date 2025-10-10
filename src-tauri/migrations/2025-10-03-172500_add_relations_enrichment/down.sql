-- Drop indexes first
DROP INDEX IF EXISTS idx_anime_relations_enriched;
DROP INDEX IF EXISTS idx_anime_relations_anime_lookup;
DROP INDEX IF EXISTS idx_anime_relations_enrichment;

-- Drop columns in reverse order
ALTER TABLE anime_relations DROP COLUMN IF EXISTS external_id;
ALTER TABLE anime_relations DROP COLUMN IF EXISTS provider_code;
ALTER TABLE anime_relations DROP COLUMN IF EXISTS last_synced;
ALTER TABLE anime_relations DROP COLUMN IF EXISTS enriched_at;
ALTER TABLE anime_relations DROP COLUMN IF EXISTS enrichment_data;
