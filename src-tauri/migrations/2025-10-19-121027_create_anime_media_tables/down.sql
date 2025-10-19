-- Drop triggers
DROP TRIGGER IF EXISTS update_anime_videos_updated_at ON anime_videos;
DROP TRIGGER IF EXISTS update_anime_images_updated_at ON anime_images;
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop tables
DROP TABLE IF EXISTS anime_videos;
DROP TABLE IF EXISTS anime_images;

-- Drop types
DROP TYPE IF EXISTS video_type;
DROP TYPE IF EXISTS image_type;
DROP TYPE IF EXISTS media_provider;
