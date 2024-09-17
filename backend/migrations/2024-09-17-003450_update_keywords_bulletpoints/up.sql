ALTER TABLE articles
    ALTER COLUMN bullet_points TYPE TEXT[] USING ARRAY[bullet_points],
    ALTER COLUMN keywords TYPE TEXT[] USING ARRAY[keywords];