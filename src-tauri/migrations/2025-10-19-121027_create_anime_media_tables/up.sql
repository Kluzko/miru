-- ============================================================================
-- CUSTOM TYPES
-- ============================================================================

CREATE TYPE media_provider AS ENUM (
    'TMDB',
    'AniList',
    'Jikan',
    'Kitsu',
    'AniDB'
);

CREATE TYPE image_type AS ENUM (
    'poster',
    'backdrop',
    'logo',
    'still',
    'banner',
    'cover'
);

CREATE TYPE video_type AS ENUM (
    'trailer',
    'teaser',
    'clip',
    'opening',
    'ending',
    'pv',
    'cm',
    'behind_the_scenes',
    'featurette'
);

-- ============================================================================
-- ANIME IMAGES TABLE
-- ============================================================================

CREATE TABLE anime_images (
    -- Primary Key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign Key
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,

    -- Source Information
    provider media_provider NOT NULL,
    provider_image_id TEXT, -- Original ID from provider (e.g., TMDB image path)

    -- Classification
    image_type image_type NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT false,

    -- Image Data
    url TEXT NOT NULL,
    width INTEGER CHECK (width IS NULL OR width > 0),
    height INTEGER CHECK (height IS NULL OR height > 0),
    aspect_ratio REAL GENERATED ALWAYS AS (
        CASE
            WHEN width IS NOT NULL AND height IS NOT NULL AND height > 0
            THEN CAST(width AS REAL) / height
            ELSE NULL
        END
    ) STORED,

    -- Quality Metrics
    vote_average REAL CHECK (vote_average IS NULL OR (vote_average >= 0 AND vote_average <= 10)),
    vote_count INTEGER CHECK (vote_count IS NULL OR vote_count >= 0),

    -- Metadata
    language VARCHAR(10),
    file_size_bytes BIGINT CHECK (file_size_bytes IS NULL OR file_size_bytes > 0),

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    synced_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT unique_provider_image UNIQUE(anime_id, provider, provider_image_id)
);

-- Partial unique index for primary images (one primary per type per anime)
CREATE UNIQUE INDEX unique_primary_image_per_type
    ON anime_images(anime_id, image_type)
    WHERE is_primary = true;

-- ============================================================================
-- ANIME VIDEOS TABLE
-- ============================================================================

CREATE TABLE anime_videos (
    -- Primary Key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign Key
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,

    -- Source Information
    provider media_provider NOT NULL,
    provider_video_id TEXT, -- Original ID from provider

    -- Classification
    video_type video_type NOT NULL,
    is_official BOOLEAN NOT NULL DEFAULT false,

    -- Video Data
    name TEXT NOT NULL,
    site VARCHAR(50) NOT NULL, -- 'YouTube', 'Vimeo', 'Dailymotion'
    key TEXT NOT NULL, -- YouTube video ID, Vimeo ID, etc.
    url TEXT NOT NULL,

    -- Quality Metrics
    resolution INTEGER CHECK (resolution IS NULL OR resolution > 0), -- 1080, 720, 480
    duration_seconds INTEGER CHECK (duration_seconds IS NULL OR duration_seconds > 0),

    -- Metadata
    language VARCHAR(10),
    published_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    synced_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT unique_provider_video UNIQUE(anime_id, provider, provider_video_id),
    CONSTRAINT unique_video_key UNIQUE(anime_id, site, key)
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Images Indexes
CREATE INDEX idx_anime_images_anime_id ON anime_images(anime_id);
CREATE INDEX idx_anime_images_type ON anime_images(anime_id, image_type);
CREATE INDEX idx_anime_images_primary ON anime_images(anime_id, is_primary)
    WHERE is_primary = true;
CREATE INDEX idx_anime_images_provider ON anime_images(provider);
CREATE INDEX idx_anime_images_quality ON anime_images(vote_average DESC NULLS LAST)
    WHERE vote_average IS NOT NULL;
CREATE INDEX idx_anime_images_url ON anime_images USING hash(url);

-- Videos Indexes
CREATE INDEX idx_anime_videos_anime_id ON anime_videos(anime_id);
CREATE INDEX idx_anime_videos_type ON anime_videos(anime_id, video_type);
CREATE INDEX idx_anime_videos_official ON anime_videos(is_official)
    WHERE is_official = true;
CREATE INDEX idx_anime_videos_provider ON anime_videos(provider);
CREATE INDEX idx_anime_videos_site_key ON anime_videos(site, key);

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Auto-update updated_at column
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_anime_images_updated_at
    BEFORE UPDATE ON anime_images
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_anime_videos_updated_at
    BEFORE UPDATE ON anime_videos
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- COMMENTS (Documentation)
-- ============================================================================

COMMENT ON TABLE anime_images IS 'Stores anime images from various providers with categorization and quality metrics';
COMMENT ON TABLE anime_videos IS 'Stores anime videos (trailers, teasers, clips) from various providers';

COMMENT ON COLUMN anime_images.provider_image_id IS 'Original image ID from provider (e.g., TMDB file_path)';
COMMENT ON COLUMN anime_images.is_primary IS 'Primary image to display (one per type per anime)';
COMMENT ON COLUMN anime_images.aspect_ratio IS 'Computed column: width/height ratio';

COMMENT ON COLUMN anime_videos.key IS 'Platform-specific video identifier (YouTube ID, Vimeo ID, etc.)';
COMMENT ON COLUMN anime_videos.is_official IS 'Whether this is an official video from the studio/distributor';
