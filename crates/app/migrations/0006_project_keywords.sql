ALTER TABLE keyword_queries
ADD COLUMN notes TEXT NOT NULL DEFAULT '';

UPDATE keyword_queries
SET notes = COALESCE(
    (
        SELECT tracked.notes
        FROM app_keywords tracked
        WHERE tracked.query_id = keyword_queries.id
          AND trim(tracked.notes) <> ''
        ORDER BY tracked.created_at, tracked.app_id
        LIMIT 1
    ),
    ''
);

DELETE FROM keyword_queries
WHERE NOT EXISTS (
    SELECT 1
    FROM app_keywords tracked
    WHERE tracked.query_id = keyword_queries.id
);

DROP INDEX app_keywords_by_query;
DROP TABLE app_keywords;
