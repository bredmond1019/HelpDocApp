-- Remove indexes from collections table
DROP INDEX IF EXISTS idx_collections_keywords_embedding;
DROP INDEX IF EXISTS idx_collections_bullet_points_embedding;
DROP INDEX IF EXISTS idx_collections_paragraph_description_embedding;

-- Remove indexes from articles table
DROP INDEX IF EXISTS idx_articles_keywords_embedding;
DROP INDEX IF EXISTS idx_articles_bullet_points_embedding;
DROP INDEX IF EXISTS idx_articles_paragraph_description_embedding;

-- Remove new columns from collections table
ALTER TABLE collections
DROP COLUMN IF EXISTS keywords_embedding,
DROP COLUMN IF EXISTS bullet_points_embedding,
DROP COLUMN IF EXISTS paragraph_description_embedding,
DROP COLUMN IF EXISTS keywords,
DROP COLUMN IF EXISTS bullet_points,
DROP COLUMN IF EXISTS paragraph_description;

-- Remove new columns from articles table
ALTER TABLE articles
DROP COLUMN IF EXISTS keywords_embedding,
DROP COLUMN IF EXISTS bullet_points_embedding,
DROP COLUMN IF EXISTS paragraph_description_embedding,
DROP COLUMN IF EXISTS keywords,
DROP COLUMN IF EXISTS bullet_points,
DROP COLUMN IF EXISTS paragraph_description;