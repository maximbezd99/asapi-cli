PRAGMA foreign_keys = ON;

CREATE TABLE project (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
    created_at TEXT NOT NULL
);

CREATE TABLE apps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    apple_id INTEGER NOT NULL UNIQUE CHECK (apple_id > 0),
    created_at TEXT NOT NULL
);

CREATE TABLE app_storefronts (
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    country TEXT NOT NULL CHECK (
        length(country) = 2
        AND country = lower(country)
    ),
    is_main INTEGER NOT NULL DEFAULT 0 CHECK (is_main IN (0, 1)),
    auto_refresh INTEGER NOT NULL DEFAULT 0 CHECK (auto_refresh IN (0, 1)),
    created_at TEXT NOT NULL,
    PRIMARY KEY (app_id, country),
    CHECK (is_main = 0 OR auto_refresh = 1)
);

CREATE UNIQUE INDEX one_main_storefront_per_app
    ON app_storefronts(app_id)
    WHERE is_main = 1;

CREATE INDEX auto_refresh_storefronts
    ON app_storefronts(auto_refresh, app_id, country);

CREATE TABLE app_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    country TEXT NOT NULL,
    fetched_at TEXT NOT NULL,
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    FOREIGN KEY (app_id, country)
        REFERENCES app_storefronts(app_id, country)
        ON DELETE CASCADE
);

CREATE INDEX app_snapshots_latest
    ON app_snapshots(app_id, country, fetched_at DESC);

CREATE TABLE app_resource_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    country TEXT NOT NULL,
    resource TEXT NOT NULL CHECK (resource IN ('iap', 'similar')),
    fetched_at TEXT NOT NULL,
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    FOREIGN KEY (app_id, country)
        REFERENCES app_storefronts(app_id, country)
        ON DELETE CASCADE
);

CREATE INDEX app_resource_snapshots_latest
    ON app_resource_snapshots(app_id, country, resource, fetched_at DESC);

CREATE TABLE reviews (
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    country TEXT NOT NULL,
    review_id INTEGER NOT NULL,
    author TEXT,
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    title TEXT,
    content TEXT NOT NULL,
    version TEXT,
    helpful_score INTEGER,
    helpful_vote_count INTEGER,
    updated_at TEXT,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    PRIMARY KEY (app_id, country, review_id),
    FOREIGN KEY (app_id, country)
        REFERENCES app_storefronts(app_id, country)
        ON DELETE CASCADE
);

CREATE INDEX reviews_recent
    ON reviews(app_id, country, updated_at DESC, review_id DESC);

CREATE TABLE review_page_fetches (
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    country TEXT NOT NULL,
    page INTEGER NOT NULL CHECK (page BETWEEN 1 AND 10),
    fetched_at TEXT NOT NULL,
    result_count INTEGER NOT NULL,
    PRIMARY KEY (app_id, country, page),
    FOREIGN KEY (app_id, country)
        REFERENCES app_storefronts(app_id, country)
        ON DELETE CASCADE
);

CREATE TABLE popularity_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    fetched_at TEXT NOT NULL,
    group_name TEXT,
    requested_countries_json TEXT NOT NULL CHECK (json_valid(requested_countries_json))
);

CREATE INDEX popularity_runs_latest
    ON popularity_runs(app_id, fetched_at DESC);

CREATE TABLE popularity_observations (
    run_id INTEGER NOT NULL REFERENCES popularity_runs(id) ON DELETE CASCADE,
    country TEXT NOT NULL,
    available INTEGER NOT NULL CHECK (available IN (0, 1)),
    name TEXT,
    rating REAL,
    rating_count INTEGER,
    PRIMARY KEY (run_id, country)
);

CREATE INDEX popularity_observations_country
    ON popularity_observations(country, run_id);

CREATE TABLE keyword_queries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query TEXT NOT NULL,
    normalized_query TEXT NOT NULL,
    country TEXT NOT NULL CHECK (
        length(country) = 2
        AND country = lower(country)
    ),
    created_at TEXT NOT NULL,
    UNIQUE (normalized_query, country)
);

CREATE TABLE app_keywords (
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    query_id INTEGER NOT NULL REFERENCES keyword_queries(id) ON DELETE CASCADE,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    PRIMARY KEY (app_id, query_id)
);

CREATE INDEX app_keywords_by_query
    ON app_keywords(query_id, app_id);

CREATE TABLE keyword_query_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_id INTEGER NOT NULL REFERENCES keyword_queries(id) ON DELETE CASCADE,
    fetched_at TEXT NOT NULL,
    result_count INTEGER NOT NULL
);

CREATE INDEX keyword_query_runs_latest
    ON keyword_query_runs(query_id, fetched_at DESC);

CREATE TABLE keyword_results (
    run_id INTEGER NOT NULL REFERENCES keyword_query_runs(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position BETWEEN 1 AND 200),
    apple_id INTEGER NOT NULL CHECK (apple_id > 0),
    name TEXT NOT NULL,
    icon_url TEXT,
    developer_name TEXT,
    PRIMARY KEY (run_id, position),
    UNIQUE (run_id, apple_id)
);

CREATE INDEX keyword_results_app_position
    ON keyword_results(run_id, apple_id, position);
