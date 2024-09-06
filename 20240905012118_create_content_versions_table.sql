-- Content versions table
CREATE TABLE content_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    article_id UUID REFERENCES articles(id),
    version_number INTEGER NOT NULL,
    markdown_content TEXT,
    edited_by VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add a composite index for faster lookups
CREATE INDEX idx_content_versions_article_version ON content_versions(article_id, version_number);