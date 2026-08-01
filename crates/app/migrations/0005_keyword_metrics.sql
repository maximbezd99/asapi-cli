ALTER TABLE keyword_queries
ADD COLUMN difficulty REAL CHECK (difficulty BETWEEN 0 AND 100);

ALTER TABLE keyword_queries
ADD COLUMN popularity REAL CHECK (popularity BETWEEN 0 AND 100);
