-- Rollback: Restore the original enum with underscores
DROP TYPE unified_age_restriction CASCADE;

CREATE TYPE unified_age_restriction AS ENUM (
    'general_audiences',
    'parental_guidance_13',
    'parental_guidance_17',
    'mature',
    'explicit'
);

-- Re-add the column
ALTER TABLE anime ADD COLUMN age_restriction unified_age_restriction;
