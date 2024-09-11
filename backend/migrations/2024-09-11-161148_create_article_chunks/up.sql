-- Enable the pg_trgm extension if not already enabled
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Create the article_chunks table
CREATE TABLE article_chunks (
    id UUID PRIMARY KEY,
    article_id UUID NOT NULL,
    content TEXT NOT NULL,
    is_title BOOLEAN NOT NULL,
    embedding_id UUID
);

-- Create indexes on the articles table
CREATE INDEX idx_articles_title ON articles USING gin (title gin_trgm_ops);
CREATE INDEX idx_articles_content ON articles USING gin (markdown_content gin_trgm_ops);

-- Add a foreign key constraint (assuming the articles table exists)
ALTER TABLE article_chunks
ADD CONSTRAINT fk_article_chunks_article
FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE;