-- Enable uuid-ossp extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE embeddings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    article_id UUID NOT NULL,
    embedding_vector float4[] NOT NULL,
    FOREIGN KEY (article_id) REFERENCES articles(id)
);

-- Create an index on article_id for faster lookups
CREATE INDEX idx_embeddings_article_id ON embeddings(article_id);

