
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Anime table
CREATE TABLE anime (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    mal_id INTEGER UNIQUE,
    title VARCHAR(255) NOT NULL,
    title_english VARCHAR(255),
    title_japanese VARCHAR(255),
    score REAL,
    scored_by INTEGER,
    rank INTEGER,
    popularity INTEGER,
    members INTEGER,
    favorites INTEGER,
    synopsis TEXT,
    episodes INTEGER,
    status VARCHAR(50) NOT NULL DEFAULT 'unknown',
    aired_from TIMESTAMPTZ,
    aired_to TIMESTAMPTZ,
    anime_type VARCHAR(50) NOT NULL DEFAULT 'Unknown',
    rating VARCHAR(50),
    source VARCHAR(100),
    duration VARCHAR(50),
    image_url TEXT,
    mal_url TEXT,
    composite_score REAL NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Genres table
CREATE TABLE genres (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    mal_id INTEGER UNIQUE,
    name VARCHAR(100) NOT NULL UNIQUE
);

-- Anime-Genre relationship
CREATE TABLE anime_genres (
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    genre_id UUID NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    PRIMARY KEY (anime_id, genre_id)
);

-- Studios table
CREATE TABLE studios (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE
);

-- Anime-Studio relationship
CREATE TABLE anime_studios (
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    studio_id UUID NOT NULL REFERENCES studios(id) ON DELETE CASCADE,
    PRIMARY KEY (anime_id, studio_id)
);

-- Collections table
CREATE TABLE collections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Collection-Anime relationship
CREATE TABLE collection_anime (
    collection_id UUID NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_score REAL CHECK (user_score >= 0 AND user_score <= 10),
    notes TEXT,
    PRIMARY KEY (collection_id, anime_id)
);

-- Quality metrics table
CREATE TABLE quality_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    anime_id UUID NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    popularity_score REAL NOT NULL DEFAULT 0.0,
    engagement_score REAL NOT NULL DEFAULT 0.0,
    consistency_score REAL NOT NULL DEFAULT 0.0,
    audience_reach_score REAL NOT NULL DEFAULT 0.0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(anime_id)
);

-- Create indexes for performance
CREATE INDEX idx_anime_mal_id ON anime(mal_id);
CREATE INDEX idx_anime_title ON anime(title);
CREATE INDEX idx_anime_title_english ON anime(title_english);
CREATE INDEX idx_anime_title_japanese ON anime(title_japanese);
CREATE INDEX idx_anime_composite_score ON anime(composite_score DESC);
CREATE INDEX idx_anime_popularity ON anime(popularity);
CREATE INDEX idx_anime_members ON anime(members DESC);
CREATE INDEX idx_anime_status ON anime(status);
CREATE INDEX idx_anime_type ON anime(anime_type);
CREATE INDEX idx_anime_aired_from ON anime(aired_from);

CREATE INDEX idx_genres_name ON genres(name);
CREATE INDEX idx_genres_mal_id ON genres(mal_id);

CREATE INDEX idx_anime_genres_anime_id ON anime_genres(anime_id);
CREATE INDEX idx_anime_genres_genre_id ON anime_genres(genre_id);

CREATE INDEX idx_studios_name ON studios(name);

CREATE INDEX idx_anime_studios_anime_id ON anime_studios(anime_id);
CREATE INDEX idx_anime_studios_studio_id ON anime_studios(studio_id);

CREATE INDEX idx_collections_name ON collections(name);
CREATE INDEX idx_collections_created_at ON collections(created_at DESC);

CREATE INDEX idx_collection_anime_collection_id ON collection_anime(collection_id);
CREATE INDEX idx_collection_anime_anime_id ON collection_anime(anime_id);
CREATE INDEX idx_collection_anime_added_at ON collection_anime(added_at DESC);

CREATE INDEX idx_quality_metrics_anime_id ON quality_metrics(anime_id);

-- Create triggers for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_anime_updated_at BEFORE UPDATE ON anime
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_collections_updated_at BEFORE UPDATE ON collections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_quality_metrics_updated_at BEFORE UPDATE ON quality_metrics
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
