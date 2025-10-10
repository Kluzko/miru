-- Add semantic franchise category variants to the anime_relation_type enum
-- These represent absolute categorization across the entire franchise

ALTER TYPE anime_relation_type ADD VALUE IF NOT EXISTS 'main_story';
ALTER TYPE anime_relation_type ADD VALUE IF NOT EXISTS 'movies';
ALTER TYPE anime_relation_type ADD VALUE IF NOT EXISTS 'ova_special';
