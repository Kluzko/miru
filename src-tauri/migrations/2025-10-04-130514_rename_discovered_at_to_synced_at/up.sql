-- Rename discovered_at to synced_at for better semantic meaning
ALTER TABLE anime_relations RENAME COLUMN discovered_at TO synced_at;

-- Update comment to reflect the new name
COMMENT ON COLUMN anime_relations.synced_at IS 'When this relation was last synchronized/updated from external providers';
