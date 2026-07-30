ALTER TABLE keyword_results
ADD COLUMN released_at TEXT;

ALTER TABLE keyword_results
ADD COLUMN version_released_at TEXT;

ALTER TABLE keyword_results
ADD COLUMN rating REAL;

ALTER TABLE keyword_results
ADD COLUMN rating_count INTEGER;
