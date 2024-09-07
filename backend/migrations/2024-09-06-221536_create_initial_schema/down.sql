-- This file should undo anything in `up.sql`
-- Drop indexes
DROP INDEX IF EXISTS idx_content_versions_article_version;
DROP INDEX IF EXISTS idx_articles_helpscout_id;
DROP INDEX IF EXISTS idx_articles_collection_id;
DROP INDEX IF EXISTS idx_articles_slug;
DROP INDEX IF EXISTS idx_collections_helpscout_id;
DROP INDEX IF EXISTS idx_collections_slug;

-- Drop tables
DROP TABLE IF EXISTS content_versions;
DROP TABLE IF EXISTS articles;
DROP TABLE IF EXISTS collections;

-- Disable uuid-ossp extension
DROP EXTENSION IF EXISTS "uuid-ossp";
