-- Remove the unique constraint on genre name
ALTER TABLE genres DROP CONSTRAINT IF EXISTS genres_name_unique;
