-- Drop all triggers first
DROP TRIGGER IF EXISTS update_anime_updated_at ON anime;
DROP TRIGGER IF EXISTS update_user_anime_data_updated_at ON user_anime_data;
DROP TRIGGER IF EXISTS update_collections_updated_at ON collections;

-- Drop functions
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop tables in dependency order
DROP TABLE IF EXISTS collection_anime CASCADE;
DROP TABLE IF EXISTS collections CASCADE;
DROP TABLE IF EXISTS user_anime_data CASCADE;
DROP TABLE IF EXISTS anime_relations CASCADE;
DROP TABLE IF EXISTS anime_external_ids CASCADE;
DROP TABLE IF EXISTS anime_studios CASCADE;
DROP TABLE IF EXISTS studios CASCADE;
DROP TABLE IF EXISTS anime_genres CASCADE;
DROP TABLE IF EXISTS genres CASCADE;
DROP TABLE IF EXISTS anime CASCADE;
DROP TABLE IF EXISTS providers CASCADE;

-- Drop enum types
DROP TYPE IF EXISTS unified_age_restriction;
DROP TYPE IF EXISTS anime_relation_type;
DROP TYPE IF EXISTS anime_tier;
DROP TYPE IF EXISTS anime_type;
DROP TYPE IF EXISTS anime_status;
DROP TYPE IF EXISTS watching_status;

-- Note: Extensions are not dropped as they might be used by other schemas
-- DROP EXTENSION IF EXISTS pg_trgm;
-- DROP EXTENSION IF EXISTS "uuid-ossp";
