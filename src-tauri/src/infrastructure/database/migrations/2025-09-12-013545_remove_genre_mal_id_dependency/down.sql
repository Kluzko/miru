-- Rollback: Add back mal_id to genres table (for emergency rollback only)
ALTER TABLE genres ADD COLUMN mal_id INTEGER;
