-- Add migration script here
CREATE TABLE articles (
    id SERIAL PRIMARY KEY,
    collection_id INTEGER REFERENCES collections(id),
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    html_content TEXT,
    markdown_content TEXT,
    version INTEGER DEFAULT 1,
    last_edited_by VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add indexes for faster lookups
CREATE INDEX idx_articles_slug ON articles(slug);
CREATE INDEX idx_articles_collection_id ON articles(collection_id);