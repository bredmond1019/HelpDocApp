ALTER TABLE articles
    ALTER COLUMN bullet_points TYPE TEXT USING bullet_points::TEXT,
    ALTER COLUMN keywords TYPE TEXT USING keywords::TEXT;