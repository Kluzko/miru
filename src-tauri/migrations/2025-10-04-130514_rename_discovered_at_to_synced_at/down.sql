-- Rename synced_at back to discovered_at
ALTER TABLE anime_relations RENAME COLUMN synced_at TO discovered_at;

-- Restore original comment
COMMENT ON COLUMN anime_relations.discovered_at IS 'When this relation was discovered/last updated from external providers';
