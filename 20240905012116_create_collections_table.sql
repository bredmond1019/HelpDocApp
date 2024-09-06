-- Create collections table
CREATE TABLE collections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    slug VARCHAR(255) UNIQUE NOT NULL,
    helpscout_collection_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add an index on the slug for faster lookups
CREATE INDEX idx_collections_slug ON collections(slug);

-- Create index on helpscout_collection_id for faster lookups
CREATE INDEX idx_collections_helpscout_id ON collections(helpscout_collection_id);