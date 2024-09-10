-- Enable the vector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Alter the embeddings table to use the vector type
ALTER TABLE embeddings
ALTER COLUMN embedding_vector TYPE vector(384) USING embedding_vector::vector(384);

-- Create an index for similarity search
CREATE INDEX ON embeddings USING ivfflat (embedding_vector vector_cosine_ops) WITH (lists = 100);