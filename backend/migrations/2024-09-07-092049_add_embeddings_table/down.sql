-- Drop the index on article_id
DROP INDEX IF EXISTS idx_embeddings_article_id;

-- Drop the embeddings table
DROP TABLE IF EXISTS embeddings;

