-- Round all existing scores to 2 decimal places
-- This fixes the precision issue for anime already in the database

UPDATE anime
SET score = ROUND(score::numeric, 2)::real
WHERE score IS NOT NULL;

UPDATE anime
SET composite_score = ROUND(composite_score::numeric, 2)::real;
