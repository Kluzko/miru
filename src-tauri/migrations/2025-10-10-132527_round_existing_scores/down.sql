-- This migration cannot be rolled back as we lose precision data
-- The original unrounded values are not stored anywhere
-- This is a one-way data transformation

SELECT 'Cannot rollback score rounding - precision data is lost' AS warning;
