-- ============================================================================
-- CREATE POSTGRESQL ENUM TYPES
-- ============================================================================

CREATE TYPE watching_status AS ENUM (
    'plan_to_watch',
    'watching',
    'completed',
    'on_hold',
    'dropped',
    'rewatching'
);

CREATE TYPE anime_status AS ENUM (
    'airing',
    'finished',
    'not_yet_aired',
    'cancelled',
    'unknown'
);

CREATE TYPE anime_type AS ENUM (
    'tv',
    'movie',
    'ova',
    'ona',
    'special',
    'music',
    'unknown'
);

CREATE TYPE anime_tier AS ENUM (
    's',
    'a',
    'b',
    'c',
    'd'
);

CREATE TYPE anime_relation_type AS ENUM (
    'sequel',
    'prequel',
    'side_story',
    'spin_off',
    'alternative',
    'summary',
    'special',
    'movie',
    'parent_story',
    'full_story',
    'same_setting',
    'shared_character',
    'other'
);

CREATE TYPE unified_age_restriction AS ENUM (
    'general_audiences',
    'parental_guidance_13',
    'parental_guidance_17',
    'mature',
    'explicit'
);

-- ============================================================================
-- TRANSFORM EXISTING ANIME TABLE
-- ============================================================================

-- Add new columns to existing anime table
ALTER TABLE anime ADD COLUMN IF NOT EXISTS title_main VARCHAR(255);
ALTER TABLE anime ADD COLUMN IF NOT EXISTS title_romaji VARCHAR(255);
ALTER TABLE anime ADD COLUMN IF NOT EXISTS title_native VARCHAR(255);
ALTER TABLE anime ADD COLUMN IF NOT EXISTS title_synonyms JSONB DEFAULT '[]';
ALTER TABLE anime ADD COLUMN IF NOT EXISTS banner_image TEXT;
ALTER TABLE anime ADD COLUMN IF NOT EXISTS trailer_url TEXT;
ALTER TABLE anime ADD COLUMN IF NOT EXISTS tier anime_tier DEFAULT 'c';
ALTER TABLE anime ADD COLUMN IF NOT EXISTS quality_metrics JSONB DEFAULT '{}';
ALTER TABLE anime ADD COLUMN IF NOT EXISTS age_restriction unified_age_restriction;

-- Populate title_main from existing title column
UPDATE anime SET title_main = title WHERE title_main IS NULL;
ALTER TABLE anime ALTER COLUMN title_main SET NOT NULL;

-- Add temporary columns for enum conversion
ALTER TABLE anime ADD COLUMN status_new anime_status;
ALTER TABLE anime ADD COLUMN anime_type_new anime_type;

-- Convert existing status values to new enum
UPDATE anime SET status_new =
    CASE LOWER(status)
        WHEN 'currently airing' THEN 'airing'::anime_status
        WHEN 'finished airing' THEN 'finished'::anime_status
        WHEN 'not yet aired' THEN 'not_yet_aired'::anime_status
        WHEN 'cancelled' THEN 'cancelled'::anime_status
        ELSE 'unknown'::anime_status
    END;

-- Convert existing anime_type values to new enum
UPDATE anime SET anime_type_new =
    CASE LOWER(anime_type)
        WHEN 'tv' THEN 'tv'::anime_type
        WHEN 'movie' THEN 'movie'::anime_type
        WHEN 'ova' THEN 'ova'::anime_type
        WHEN 'ona' THEN 'ona'::anime_type
        WHEN 'special' THEN 'special'::anime_type
        WHEN 'music' THEN 'music'::anime_type
        ELSE 'unknown'::anime_type
    END;

-- Drop old columns and rename new ones
ALTER TABLE anime DROP COLUMN status;
ALTER TABLE anime DROP COLUMN anime_type;
ALTER TABLE anime DROP COLUMN title;
ALTER TABLE anime RENAME COLUMN status_new TO status;
ALTER TABLE anime RENAME COLUMN anime_type_new TO anime_type;

-- Set NOT NULL constraints
ALTER TABLE anime ALTER COLUMN status SET NOT NULL;
ALTER TABLE anime ALTER COLUMN anime_type SET NOT NULL;
ALTER TABLE anime ALTER COLUMN tier SET NOT NULL;

-- Convert rating column to age_restriction if exists
UPDATE anime SET age_restriction =
    CASE
        WHEN rating ILIKE '%G%' OR rating ILIKE '%All Ages%' THEN 'general_audiences'::unified_age_restriction
        WHEN rating ILIKE '%PG-13%' OR rating ILIKE '%13%' THEN 'parental_guidance_13'::unified_age_restriction
        WHEN rating ILIKE '%PG-17%' OR rating ILIKE '%17%' THEN 'parental_guidance_17'::unified_age_restriction
        WHEN rating ILIKE '%R%' OR rating ILIKE '%Mature%' THEN 'mature'::unified_age_restriction
        WHEN rating ILIKE '%X%' OR rating ILIKE '%Adult%' THEN 'explicit'::unified_age_restriction
        ELSE NULL
    END
WHERE rating IS NOT NULL;

-- ============================================================================
-- UPDATE EXISTING TABLES AND CREATE NEW ONES
-- ============================================================================

-- Update genres table to add mal_id if it doesn't exist
ALTER TABLE genres ADD COLUMN IF NOT EXISTS mal_id INTEGER UNIQUE;
ALTER TABLE genres ALTER COLUMN name SET NOT NULL;

-- anime_genres table already exists, no changes needed

-- studios table already exists, no changes needed

-- anime_studios table already exists, no changes needed

-- Provider registry (new table)
CREATE TABLE IF NOT EXISTS providers (
    code VARCHAR(20) PRIMARY KEY,
    display_name VARCHAR(100) NOT NULL,
    api_base_url VARCHAR(255),
    is_active BOOLEAN DEFAULT true
);

-- External provider ID mappings (new table)
CREATE TABLE IF NOT EXISTS anime_external_ids (
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    provider_code VARCHAR(20) NOT NULL REFERENCES providers(code),
    external_id VARCHAR(255) NOT NULL,
    provider_url TEXT,
    is_primary BOOLEAN DEFAULT false,
    last_synced TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (anime_id, provider_code),
    UNIQUE (provider_code, external_id)
);

-- Anime relationships (new table)
CREATE TABLE IF NOT EXISTS anime_relations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    related_anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    relation_type anime_relation_type NOT NULL,
    order_index INTEGER CHECK (order_index >= 0),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (anime_id, related_anime_id, relation_type),
    CHECK (anime_id != related_anime_id)
);

-- User anime tracking data (new table)
CREATE TABLE IF NOT EXISTS user_anime_data (
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL,

    -- NULL = not in user's tracking list (neutral state)
    -- Non-NULL = user has added anime with specific status
    status watching_status,

    personal_rating REAL CHECK (personal_rating >= 0 AND personal_rating <= 10),
    episodes_watched INTEGER DEFAULT 0 CHECK (episodes_watched >= 0),
    rewatched_count INTEGER DEFAULT 0 CHECK (rewatched_count >= 0),
    is_favorite BOOLEAN DEFAULT false,

    -- User notes and tags
    notes TEXT,
    tags JSONB DEFAULT '[]',

    -- Tracking dates
    start_date TIMESTAMPTZ,
    finish_date TIMESTAMPTZ,
    CHECK (start_date IS NULL OR finish_date IS NULL OR finish_date >= start_date),

    -- Audit timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (anime_id, user_id)
);

-- Update existing collections table
ALTER TABLE collections ADD COLUMN IF NOT EXISTS user_id VARCHAR(255);
ALTER TABLE collections ADD COLUMN IF NOT EXISTS is_public BOOLEAN DEFAULT false;

-- Ensure unique collection names per user (drop existing constraint first if needed)
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'ux_collections_name_lower'
        AND table_name = 'collections'
    ) THEN
        DROP INDEX ux_collections_name_lower;
    END IF;
END $$;

-- Add new unique constraint if user_id column was added
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'collections_user_id_name_key'
        AND table_name = 'collections'
    ) THEN
        ALTER TABLE collections ADD CONSTRAINT collections_user_id_name_key UNIQUE (user_id, name);
    END IF;
END $$;

-- collection_anime table already exists, just add missing columns if needed
ALTER TABLE collection_anime ADD COLUMN IF NOT EXISTS added_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP;

-- ============================================================================
-- CREATE PERFORMANCE INDEXES (IF NOT EXISTS)
-- ============================================================================

-- Core anime indexes (create only new ones)
CREATE INDEX IF NOT EXISTS idx_anime_tier ON anime(tier);
CREATE INDEX IF NOT EXISTS idx_anime_title_main_trgm ON anime USING gin(LOWER(title_main) gin_trgm_ops);

-- External ID indexes (new tables)
CREATE INDEX IF NOT EXISTS idx_anime_external_ids_lookup ON anime_external_ids(provider_code, external_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_anime_external_ids_primary
    ON anime_external_ids(anime_id) WHERE is_primary = true;

-- Relation indexes (new tables)
CREATE INDEX IF NOT EXISTS idx_anime_relations_anime_id ON anime_relations(anime_id, relation_type);
CREATE INDEX IF NOT EXISTS idx_anime_relations_related_id ON anime_relations(related_anime_id);

-- User data indexes (new tables)
CREATE INDEX IF NOT EXISTS idx_user_anime_user_status ON user_anime_data(user_id, status);
CREATE INDEX IF NOT EXISTS idx_user_anime_favorites ON user_anime_data(user_id, is_favorite) WHERE is_favorite = true;
CREATE INDEX IF NOT EXISTS idx_user_anime_updated ON user_anime_data(updated_at DESC);

-- Collection indexes (for new columns)
CREATE INDEX IF NOT EXISTS idx_collections_user ON collections(user_id);
CREATE INDEX IF NOT EXISTS idx_collections_public ON collections(is_public) WHERE is_public = true;

-- ============================================================================
-- CREATE TRIGGERS AND FUNCTIONS (IF NOT EXISTS)
-- ============================================================================

-- Apply updated_at triggers to new tables only
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.triggers
        WHERE trigger_name = 'update_user_anime_data_updated_at'
        AND event_object_table = 'user_anime_data'
    ) THEN
        CREATE TRIGGER update_user_anime_data_updated_at
            BEFORE UPDATE ON user_anime_data
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;

-- ============================================================================
-- INSERT DEFAULT DATA
-- ============================================================================

-- Insert default providers (using ON CONFLICT to avoid errors)
INSERT INTO providers (code, display_name, api_base_url) VALUES
    ('jikan', 'MyAnimeList (Jikan)', 'https://api.jikan.moe/v4'),
    ('anilist', 'AniList', 'https://graphql.anilist.co'),
    ('kitsu', 'Kitsu', 'https://kitsu.io/api/edge')
ON CONFLICT (code) DO NOTHING;

-- ============================================================================
-- ADD HELPFUL COMMENTS
-- ============================================================================

COMMENT ON TABLE anime IS 'Main anime information table with unified approach';
COMMENT ON COLUMN anime.status IS 'Current airing status of the anime';
COMMENT ON COLUMN anime.tier IS 'Internal quality tier (S=best, D=worst)';
COMMENT ON COLUMN anime.quality_metrics IS 'JSONB containing calculated quality metrics';

COMMENT ON TABLE user_anime_data IS 'User tracking data for anime';
COMMENT ON COLUMN user_anime_data.status IS 'NULL = not tracked, non-NULL = specific tracking status';

COMMENT ON TABLE anime_external_ids IS 'Mapping between internal anime and external provider IDs';
COMMENT ON INDEX idx_anime_external_ids_primary IS 'Ensures only one primary provider per anime';
