-- ============================================================================
-- RESTORE ANIME_RELATIONS TABLE TO PREVIOUS STATE
-- ============================================================================
-- This reverses the simplification, adding back the removed columns

-- Rename discovered_at back to last_synced
ALTER TABLE anime_relations RENAME COLUMN discovered_at TO last_synced;

-- Add back the removed columns
ALTER TABLE anime_relations ADD COLUMN order_index INTEGER CHECK (order_index >= 0);
ALTER TABLE anime_relations ADD COLUMN enrichment_data JSONB NOT NULL DEFAULT '{}';
ALTER TABLE anime_relations ADD COLUMN enriched_at TIMESTAMPTZ;
ALTER TABLE anime_relations ADD COLUMN provider_code VARCHAR(20);
ALTER TABLE anime_relations ADD COLUMN external_id VARCHAR(255);

-- Restore foreign key constraint
ALTER TABLE anime_relations ADD CONSTRAINT anime_relations_provider_code_fkey
    FOREIGN KEY (provider_code) REFERENCES providers(code);

-- Restore indexes
CREATE INDEX idx_anime_relations_enrichment
    ON anime_relations USING gin(enrichment_data);

CREATE INDEX idx_anime_relations_enriched
    ON anime_relations(enriched_at) WHERE enriched_at IS NOT NULL;
