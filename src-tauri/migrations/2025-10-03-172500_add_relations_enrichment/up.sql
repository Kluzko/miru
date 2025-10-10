-- Add JSONB enrichment column to existing anime_relations table
ALTER TABLE anime_relations
ADD COLUMN enrichment_data JSONB NOT NULL DEFAULT '{}';

-- Add enrichment timestamp
ALTER TABLE anime_relations
ADD COLUMN enriched_at TIMESTAMPTZ;

-- Add last synced timestamp
ALTER TABLE anime_relations
ADD COLUMN last_synced TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP;

-- Add provider info
ALTER TABLE anime_relations
ADD COLUMN provider_code VARCHAR(20) REFERENCES providers(code);

-- Add external ID for tracking
ALTER TABLE anime_relations
ADD COLUMN external_id VARCHAR(255);

-- Create index for fast JSONB queries
CREATE INDEX idx_anime_relations_enrichment
ON anime_relations USING gin(enrichment_data);

-- Create index for anime_id lookups
CREATE INDEX idx_anime_relations_anime_lookup
ON anime_relations(anime_id, relation_type);

-- Create index for enrichment status
CREATE INDEX idx_anime_relations_enriched
ON anime_relations(enriched_at) WHERE enriched_at IS NOT NULL;
