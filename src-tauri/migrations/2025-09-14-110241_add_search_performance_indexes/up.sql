-- Add critical indexes for search performance optimization
-- These indexes will dramatically improve anime search and lookup performance
-- Note: CONCURRENTLY removed for compatibility with Diesel transaction-based migrations
-- Using IF NOT EXISTS to handle potentially existing indexes

-- 1. Composite index for multi-field title search (most critical)
CREATE INDEX IF NOT EXISTS idx_anime_title_search
ON anime (title_main, title_english, title_japanese);

-- 2. GIN index for synonyms JSON array search (critical for enhanced search)
CREATE INDEX IF NOT EXISTS idx_anime_synonyms_gin
ON anime USING GIN (title_synonyms);

-- 3. Index for composite score ordering (frequently used in search results)
CREATE INDEX IF NOT EXISTS idx_anime_composite_score_desc
ON anime (composite_score DESC);

-- 4. Index for external ID lookups (critical for duplicate detection)
CREATE INDEX IF NOT EXISTS idx_anime_external_ids_lookup
ON anime_external_ids (provider_code, external_id);

-- 5. Index for external ID to anime mapping (reverse lookup)
CREATE INDEX IF NOT EXISTS idx_anime_external_ids_anime
ON anime_external_ids (anime_id);

-- 6. Composite index for collection anime operations
CREATE INDEX IF NOT EXISTS idx_collection_anime_lookup
ON collection_anime (collection_id, added_at DESC);

-- 7. Index for anime genre filtering
CREATE INDEX IF NOT EXISTS idx_anime_genres_anime
ON anime_genres (anime_id);

-- 8. Index for anime studio filtering
CREATE INDEX IF NOT EXISTS idx_anime_studios_anime
ON anime_studios (anime_id);

-- 9. Partial index for non-null scores (common filter)
CREATE INDEX IF NOT EXISTS idx_anime_score_notnull
ON anime (score DESC) WHERE score IS NOT NULL;

-- 10. Index for status filtering (common in searches)
CREATE INDEX IF NOT EXISTS idx_anime_status
ON anime (status);

-- 11. Index for type filtering (common in searches)
CREATE INDEX IF NOT EXISTS idx_anime_type
ON anime (anime_type);

-- 12. Index for aired date filtering (using direct date field instead of EXTRACT)
CREATE INDEX IF NOT EXISTS idx_anime_aired_from_date
ON anime (aired_from) WHERE aired_from IS NOT NULL;
