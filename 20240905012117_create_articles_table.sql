-- Create articles table
CREATE TABLE articles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    collection_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    html_content TEXT,
    markdown_content TEXT,
    version INTEGER DEFAULT 0,
    last_edited_by VARCHAR(255),
    helpscout_collection_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (collection_id) REFERENCES collections(id)
);

-- Create index on the slug for faster lookups
CREATE INDEX idx_articles_slug ON articles(slug);

-- Create index on collection_id for faster lookups
CREATE INDEX idx_articles_collection_id ON articles(collection_id);

-- Create index on helpscout_collection_id for faster lookups
CREATE INDEX idx_articles_helpscout_id ON articles(helpscout_collection_id);