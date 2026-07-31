CREATE TABLE app_estimate_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    fetched_at TEXT NOT NULL,
    downloads_value INTEGER CHECK (downloads_value >= 0),
    downloads_rounded INTEGER CHECK (downloads_rounded >= 0),
    downloads_prefix TEXT,
    downloads_display TEXT,
    downloads_units TEXT,
    revenue_value INTEGER CHECK (revenue_value >= 0),
    revenue_rounded INTEGER CHECK (revenue_rounded >= 0),
    revenue_prefix TEXT,
    revenue_display TEXT,
    revenue_units TEXT,
    revenue_currency TEXT CHECK (
        revenue_currency IS NULL OR length(revenue_currency) = 3
    ),
    CHECK (
        (downloads_value IS NULL
            AND downloads_rounded IS NULL
            AND downloads_prefix IS NULL
            AND downloads_display IS NULL
            AND downloads_units IS NULL)
        OR
        (downloads_value IS NOT NULL
            AND downloads_rounded IS NOT NULL
            AND downloads_display IS NOT NULL
            AND downloads_units IS NOT NULL)
    ),
    CHECK (
        (revenue_value IS NULL
            AND revenue_rounded IS NULL
            AND revenue_prefix IS NULL
            AND revenue_display IS NULL
            AND revenue_units IS NULL
            AND revenue_currency IS NULL)
        OR
        (revenue_value IS NOT NULL
            AND revenue_rounded IS NOT NULL
            AND revenue_display IS NOT NULL
            AND revenue_units IS NOT NULL
            AND revenue_currency IS NOT NULL)
    )
);

CREATE INDEX app_estimate_snapshots_latest
    ON app_estimate_snapshots(app_id, fetched_at DESC);
