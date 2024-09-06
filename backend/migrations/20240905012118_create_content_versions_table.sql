-- Add migration script here
CREATE TABLE content_versions (
    id SERIAL PRIMARY KEY,
    article_id INTEGER REFERENCES articles(id),
    version_number INTEGER NOT NULL,
    markdown_content TEXT,
    edited_by VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add a composite index for faster lookups
CREATE INDEX idx_content_versions_article_version ON content_versions(article_id, version_number);