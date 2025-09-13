-- Remove MAL ID dependency from genres table
-- This makes genres provider-agnostic as they should be

-- Remove the mal_id column from genres table
ALTER TABLE genres DROP COLUMN IF EXISTS mal_id;
