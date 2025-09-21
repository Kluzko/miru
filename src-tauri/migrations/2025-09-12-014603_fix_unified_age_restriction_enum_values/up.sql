-- Fix enum values to match what Rust diesel_derive_enum generates
-- The Rust enum generates: parental_guidance13, parental_guidance17
-- But database has: parental_guidance_13, parental_guidance_17

-- Drop and recreate the enum with correct values
DROP TYPE unified_age_restriction CASCADE;

CREATE TYPE unified_age_restriction AS ENUM (
    'general_audiences',
    'parental_guidance13',
    'parental_guidance17',
    'mature',
    'explicit'
);

-- Re-add the column with the new enum
ALTER TABLE anime ADD COLUMN age_restriction unified_age_restriction;
