-- Make created_at and updated_at non-nullable in articles table
ALTER TABLE articles
ALTER COLUMN created_at SET NOT NULL,
ALTER COLUMN updated_at SET NOT NULL,
ALTER COLUMN version SET NOT NULL;

-- Make created_at and updated_at non-nullable in collections table
ALTER TABLE collections
ALTER COLUMN created_at SET NOT NULL,
ALTER COLUMN updated_at SET NOT NULL;

-- Make created_at non-nullable in content_versions table
ALTER TABLE content_versions
ALTER COLUMN created_at SET NOT NULL;

-- Set default values for existing NULL entries
UPDATE articles SET created_at = NOW() WHERE created_at IS NULL;
UPDATE articles SET updated_at = NOW() WHERE updated_at IS NULL;
UPDATE articles SET version = 1 WHERE version IS NULL;
UPDATE collections SET created_at = NOW() WHERE created_at IS NULL;
UPDATE collections SET updated_at = NOW() WHERE updated_at IS NULL;
UPDATE content_versions SET created_at = NOW() WHERE created_at IS NULL;