-- Revert changes in articles table
ALTER TABLE articles
ALTER COLUMN created_at DROP NOT NULL,
ALTER COLUMN updated_at DROP NOT NULL,
ALTER COLUMN version DROP NOT NULL;

-- Revert changes in collections table
ALTER TABLE collections
ALTER COLUMN created_at DROP NOT NULL,
ALTER COLUMN updated_at DROP NOT NULL;

-- Revert changes in content_versions table
ALTER TABLE content_versions
ALTER COLUMN created_at DROP NOT NULL;