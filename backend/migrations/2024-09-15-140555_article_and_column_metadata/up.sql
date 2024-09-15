-- Add new columns to articles table
ALTER TABLE articles
ADD COLUMN paragraph_description TEXT,
ADD COLUMN bullet_points TEXT,
ADD COLUMN keywords TEXT,
ADD COLUMN paragraph_description_embedding vector(384),
ADD COLUMN bullet_points_embedding vector(384),
ADD COLUMN keywords_embedding vector(384);

-- Add new columns to collections table
ALTER TABLE collections
ADD COLUMN paragraph_description TEXT,
ADD COLUMN bullet_points TEXT,
ADD COLUMN keywords TEXT,
ADD COLUMN paragraph_description_embedding vector(384),
ADD COLUMN bullet_points_embedding vector(384),
ADD COLUMN keywords_embedding vector(384);

-- Create indexes for the new embedding columns in articles table
CREATE INDEX idx_articles_paragraph_description_embedding ON articles USING ivfflat (paragraph_description_embedding vector_cosine_ops);
CREATE INDEX idx_articles_bullet_points_embedding ON articles USING ivfflat (bullet_points_embedding vector_cosine_ops);
CREATE INDEX idx_articles_keywords_embedding ON articles USING ivfflat (keywords_embedding vector_cosine_ops);

-- Create indexes for the new embedding columns in collections table
CREATE INDEX idx_collections_paragraph_description_embedding ON collections USING ivfflat (paragraph_description_embedding vector_cosine_ops);
CREATE INDEX idx_collections_bullet_points_embedding ON collections USING ivfflat (bullet_points_embedding vector_cosine_ops);
CREATE INDEX idx_collections_keywords_embedding ON collections USING ivfflat (keywords_embedding vector_cosine_ops);
