-- Your SQL goes here
-- Ensure pg_trgm exists (unrelated to the error, but you use similarity())
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- anime: unique on mal_id, but allow multiple NULLs (partial unique index)
CREATE UNIQUE INDEX IF NOT EXISTS idx_anime_mal_id_unique
  ON anime (mal_id)
  WHERE mal_id IS NOT NULL;

-- genres: unique on mal_id if provided
CREATE UNIQUE INDEX IF NOT EXISTS idx_genres_mal_id_unique
  ON genres (mal_id)
  WHERE mal_id IS NOT NULL;

-- genres: avoid duplicates when mal_id is NULL (by name)
CREATE UNIQUE INDEX IF NOT EXISTS idx_genres_name_unique_when_no_mal
  ON genres (name)
  WHERE mal_id IS NULL;

-- studios: unique by name
CREATE UNIQUE INDEX IF NOT EXISTS idx_studios_name_unique
  ON studios (name);

-- quality_metrics: one row per anime
CREATE UNIQUE INDEX IF NOT EXISTS idx_quality_metrics_anime_id_unique
  ON quality_metrics (anime_id);

-- junction tables: prevent duplicate links
CREATE UNIQUE INDEX IF NOT EXISTS idx_anime_genres_unique
  ON anime_genres (anime_id, genre_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_anime_studios_unique
  ON anime_studios (anime_id, studio_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_collection_anime_unique
  ON collection_anime (collection_id, anime_id);
