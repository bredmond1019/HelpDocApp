-- Drop the foreign key constraint
ALTER TABLE article_chunks
DROP CONSTRAINT IF EXISTS fk_article_chunks_article;

-- Drop the article_chunks table
DROP TABLE IF EXISTS article_chunks;

-- Drop the indexes on the articles table
DROP INDEX IF EXISTS idx_articles_title;
DROP INDEX IF EXISTS idx_articles_content;

-- Note: We don't drop the pg_trgm extension as it might be used by other parts of the application