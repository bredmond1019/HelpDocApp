-- Drop the index created for similarity search
DROP INDEX IF EXISTS embeddings_embedding_vector_idx;

-- Alter the embeddings table back to use float4[]
ALTER TABLE embeddings
ALTER COLUMN embedding_vector TYPE float4[] USING embedding_vector::float4[];

-- Drop the vector extension if it's no longer needed
-- Note: Only do this if no other tables are using the vector type
-- DROP EXTENSION IF EXISTS vector;
