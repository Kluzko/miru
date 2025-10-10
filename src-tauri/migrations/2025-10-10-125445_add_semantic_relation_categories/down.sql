-- Note: PostgreSQL does not support removing enum values directly
-- To rollback, you would need to:
-- 1. Update all rows using the new values to 'other'
-- 2. Drop and recreate the enum type
-- This is complex and risky, so we document it here but don't implement automatic rollback

-- For a proper rollback, you would need to:
-- UPDATE anime_relations SET relation_type = 'other' WHERE relation_type IN ('main_story', 'movies', 'ova_special');
-- Then recreate the enum without these values (complex process)

SELECT 'Cannot automatically rollback enum value additions in PostgreSQL' AS warning;
