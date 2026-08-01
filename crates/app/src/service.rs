use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use anyhow::{bail, Context, Result};
use appstore_api::{
    app_store::AppSpecifier,
    commands,
    countries::validate_country,
    market_estimates,
    requests::{LookupRequest, PopularityGroup, PopularityRequest, ReviewsRequest, SearchRequest},
    ApiClient, ClientConfig, Envelope,
};
use chrono::{DateTime, Utc};
use indoc::indoc;
use serde_json::{json, Value};
use sqlx::{FromRow, Row, Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    keyword::KeywordIdentity,
    manager::{ProjectHandle, ProjectManager},
    models::{
        AddKeyword, AppEstimateMetric, AppEstimatesView, AppView, HistorySnapshot, KeywordEntity,
        KeywordTrendPoint, KeywordView, PatchValue, PopularityCountry, PopularityView, RankedApp,
        Review, ReviewSummary, ReviewsPage, Storefront, TrackedAppSummary, UpdateKeywordMetrics,
        UpdateStorefront,
    },
};

const FRESH_FOR: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone)]
pub struct AppService {
    manager: ProjectManager,
    client: Arc<ApiClient>,
    keyword_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

#[derive(FromRow)]
struct AppRecord {
    id: i64,
    apple_id: i64,
    created_at: String,
}

#[derive(FromRow)]
struct AppSummaryRow {
    apple_id: i64,
    created_at: String,
    main_country: String,
    payload_json: Option<String>,
    last_updated: Option<String>,
    popularity_rating: Option<f64>,
    popularity_rating_count: Option<i64>,
}

#[derive(FromRow)]
struct KeywordRecord {
    query_id: i64,
    query: String,
    normalized_query: String,
    country: String,
    difficulty: Option<f64>,
    popularity: Option<f64>,
    notes: String,
}

#[derive(FromRow)]
struct KeywordEntityRecord {
    query_id: i64,
    query: String,
    normalized_query: String,
    country: String,
    notes: String,
    difficulty: Option<f64>,
    popularity: Option<f64>,
}

#[derive(FromRow)]
struct EstimateSnapshotRow {
    fetched_at: String,
    downloads_value: Option<i64>,
    downloads_rounded: Option<i64>,
    downloads_prefix: Option<String>,
    downloads_display: Option<String>,
    downloads_units: Option<String>,
    revenue_value: Option<i64>,
    revenue_rounded: Option<i64>,
    revenue_prefix: Option<String>,
    revenue_display: Option<String>,
    revenue_units: Option<String>,
    revenue_currency: Option<String>,
}

impl AppService {
    pub fn new(manager: ProjectManager, client_config: ClientConfig) -> Result<Self> {
        Ok(Self {
            manager,
            client: Arc::new(ApiClient::new(client_config)?),
            keyword_locks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn manager(&self) -> &ProjectManager {
        &self.manager
    }

    pub fn client(&self) -> &ApiClient {
        &self.client
    }

    pub async fn list_apps(&self, project_id: &str) -> Result<Vec<TrackedAppSummary>> {
        let project = self.manager.get(project_id).await?;
        let rows = sqlx::query_as::<_, AppSummaryRow>(indoc! {r#"
            WITH latest_popularity AS (
                SELECT runs.app_id, runs.id
                FROM popularity_runs runs
                WHERE runs.id = (
                    SELECT latest.id
                    FROM popularity_runs latest
                    WHERE latest.app_id = runs.app_id
                    ORDER BY latest.fetched_at DESC
                    LIMIT 1
                )
            ),
            popularity_totals AS (
                SELECT
                    latest.app_id,
                    SUM(
                        CASE
                            WHEN observations.available = 1
                            THEN COALESCE(observations.rating_count, 0)
                            ELSE 0
                        END
                    ) AS popularity_rating_count,
                    SUM(
                        CASE
                            WHEN observations.available = 1
                             AND observations.rating IS NOT NULL
                            THEN observations.rating
                               * COALESCE(observations.rating_count, 0)
                            ELSE 0
                        END
                    ) / NULLIF(
                        SUM(
                            CASE
                                WHEN observations.available = 1
                                 AND observations.rating IS NOT NULL
                                THEN COALESCE(observations.rating_count, 0)
                                ELSE 0
                            END
                        ),
                        0
                    ) AS popularity_rating
                FROM latest_popularity latest
                JOIN popularity_observations observations
                  ON observations.run_id = latest.id
                GROUP BY latest.app_id
            )
            SELECT
                a.apple_id,
                a.created_at,
                sf.country AS main_country,
                (
                    SELECT snapshots.payload_json
                    FROM app_snapshots snapshots
                    WHERE snapshots.app_id = a.id
                      AND snapshots.country = sf.country
                    ORDER BY snapshots.fetched_at DESC
                    LIMIT 1
                ) AS payload_json,
                (
                    SELECT snapshots.fetched_at
                    FROM app_snapshots snapshots
                    WHERE snapshots.app_id = a.id
                      AND snapshots.country = sf.country
                    ORDER BY snapshots.fetched_at DESC
                    LIMIT 1
                ) AS last_updated,
                totals.popularity_rating,
                totals.popularity_rating_count
            FROM apps a
            JOIN app_storefronts sf
              ON sf.app_id = a.id
             AND sf.is_main = 1
            LEFT JOIN popularity_totals totals
              ON totals.app_id = a.id
            ORDER BY
                COALESCE(
                    json_extract(payload_json, '$.data[0].name'),
                    CAST(a.apple_id AS TEXT)
                ) COLLATE NOCASE
        "#})
        .fetch_all(&project.pool)
        .await
        .context("failed to list apps")?;

        rows.into_iter()
            .map(|row| {
                let payload = row
                    .payload_json
                    .as_deref()
                    .map(serde_json::from_str::<Value>)
                    .transpose()?;
                let data = payload.as_ref().and_then(|value| value.pointer("/data/0"));
                let main_rating = data
                    .and_then(|value| value.get("rating"))
                    .and_then(Value::as_f64);
                let main_rating_count = data
                    .and_then(|value| value.get("rating_count"))
                    .and_then(Value::as_i64);
                Ok(TrackedAppSummary {
                    apple_id: row.apple_id,
                    created_at: row.created_at,
                    main_country: row.main_country,
                    name: string_at(data, "/name"),
                    icon_url: string_at(data, "/icon_url"),
                    rating: row.popularity_rating.or(main_rating),
                    rating_count: row
                        .popularity_rating_count
                        .filter(|count| *count > 0)
                        .or(main_rating_count),
                    version: string_at(data, "/version"),
                    last_updated: row.last_updated,
                })
            })
            .collect()
    }

    pub async fn add_app(&self, project_id: &str, app_id: u64, country: &str) -> Result<AppView> {
        let country = validate_country(country)?;
        let app_id = checked_app_id(app_id)?;
        let project = self.manager.get(project_id).await?;
        let mut tx = project.pool.begin().await?;
        let result = sqlx::query(indoc! {r#"
            INSERT INTO apps (apple_id, created_at)
            VALUES (?, ?)
        "#})
        .bind(app_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&mut *tx)
        .await
        .context("this app is already tracked in the project")?;
        let internal_id = result.last_insert_rowid();
        sqlx::query(indoc! {r#"
            INSERT INTO app_storefronts (
                app_id,
                country,
                is_main,
                auto_refresh,
                created_at
            )
            VALUES (?, ?, 1, 1, ?)
        "#})
        .bind(internal_id)
        .bind(&country)
        .bind(Utc::now().to_rfc3339())
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        if let Err(error) = self.refresh_app(project_id, app_id, Some(&country)).await {
            sqlx::query(indoc! {r#"
                DELETE FROM apps
                WHERE id = ?
            "#})
            .bind(internal_id)
            .execute(&project.pool)
            .await
            .context("failed to roll back an app whose initial refresh failed")?;
            return Err(error).context("initial App Store refresh failed; the app was not added");
        }
        self.app_view(project_id, app_id, Some(&country)).await
    }

    pub async fn delete_app(&self, project_id: &str, app_id: i64) -> Result<()> {
        let project = self.manager.get(project_id).await?;
        let result = sqlx::query(indoc! {r#"
            DELETE FROM apps
            WHERE apple_id = ?
        "#})
        .bind(app_id)
        .execute(&project.pool)
        .await?;
        if result.rows_affected() == 0 {
            bail!("app {app_id} is not tracked in this project");
        }
        Ok(())
    }

    pub async fn app_view(
        &self,
        project_id: &str,
        app_id: i64,
        country: Option<&str>,
    ) -> Result<AppView> {
        let project = self.manager.get(project_id).await?;
        let app = find_app(&project.pool, app_id).await?;
        let storefronts = self.storefronts(project_id, app_id).await?;
        let selected_country = match country {
            Some(country) => validate_country(country)?,
            None => storefronts
                .iter()
                .find(|storefront| storefront.is_main)
                .map(|storefront| storefront.country.clone())
                .context("tracked app has no main storefront")?,
        };
        require_storefront(&project.pool, app.id, &selected_country).await?;

        let details = latest_payload(
            &project.pool,
            indoc! {r#"
                SELECT payload_json, fetched_at
                FROM app_snapshots
                WHERE app_id = ? AND country = ?
                ORDER BY fetched_at DESC
                LIMIT 1
            "#},
            app.id,
            &selected_country,
            None,
        )
        .await?;
        let iap = latest_payload(
            &project.pool,
            indoc! {r#"
                SELECT payload_json, fetched_at
                FROM app_resource_snapshots
                WHERE app_id = ? AND country = ? AND resource = ?
                ORDER BY fetched_at DESC
                LIMIT 1
            "#},
            app.id,
            &selected_country,
            Some("iap"),
        )
        .await?;
        let similar = latest_payload(
            &project.pool,
            indoc! {r#"
                SELECT payload_json, fetched_at
                FROM app_resource_snapshots
                WHERE app_id = ? AND country = ? AND resource = ?
                ORDER BY fetched_at DESC
                LIMIT 1
            "#},
            app.id,
            &selected_country,
            Some("similar"),
        )
        .await?;
        let estimates = latest_app_estimates(&project.pool, app.id).await?;
        let popularity = latest_popularity(&project.pool, app.id).await?;
        let review_summary = review_summary(&project.pool, app.id, &selected_country).await?;

        Ok(AppView {
            apple_id: app.apple_id,
            created_at: app.created_at,
            selected_country,
            storefronts,
            details: details.as_ref().map(|(payload, _)| payload.clone()),
            details_updated_at: details.map(|(_, fetched_at)| fetched_at),
            estimates,
            iap: iap.map(|(payload, _)| payload),
            similar: similar.map(|(payload, _)| payload),
            popularity,
            review_summary,
        })
    }

    pub async fn storefronts(&self, project_id: &str, app_id: i64) -> Result<Vec<Storefront>> {
        let project = self.manager.get(project_id).await?;
        let app = find_app(&project.pool, app_id).await?;
        sqlx::query_as::<_, Storefront>(indoc! {r#"
            SELECT
                country,
                is_main,
                auto_refresh,
                created_at
            FROM app_storefronts
            WHERE app_id = ?
            ORDER BY is_main DESC, country
        "#})
        .bind(app.id)
        .fetch_all(&project.pool)
        .await
        .context("failed to list app storefronts")
    }

    pub async fn app_history(
        &self,
        project_id: &str,
        app_id: i64,
        country: Option<&str>,
        resource: &str,
    ) -> Result<Vec<HistorySnapshot>> {
        let project = self.manager.get(project_id).await?;
        let app = find_app(&project.pool, app_id).await?;
        if resource == "popularity" {
            return popularity_history(&project.pool, app.id).await;
        }
        if resource == "estimates" {
            return estimate_history(&project.pool, app.id).await;
        }
        let country = match country {
            Some(country) => validate_country(country)?,
            None => {
                sqlx::query_scalar(indoc! {r#"
                SELECT country
                FROM app_storefronts
                WHERE app_id = ? AND is_main = 1
            "#})
                .bind(app.id)
                .fetch_one(&project.pool)
                .await?
            }
        };
        require_storefront(&project.pool, app.id, &country).await?;
        let rows = match resource {
            "details" => {
                sqlx::query(indoc! {r#"
                    SELECT fetched_at, payload_json
                    FROM app_snapshots
                    WHERE app_id = ?
                      AND country = ?
                      AND datetime(fetched_at) >= datetime('now', '-30 days')
                    ORDER BY fetched_at DESC
                "#})
                .bind(app.id)
                .bind(&country)
                .fetch_all(&project.pool)
                .await?
            }
            _ => bail!(
                "unsupported history resource '{resource}'; use details, estimates, or popularity"
            ),
        };
        rows.into_iter()
            .map(|row| {
                let payload: String = row.try_get("payload_json")?;
                Ok(HistorySnapshot {
                    resource: resource.to_string(),
                    country: Some(country.clone()),
                    fetched_at: row.try_get("fetched_at")?,
                    payload: serde_json::from_str(&payload)?,
                })
            })
            .collect()
    }

    pub async fn add_storefront(
        &self,
        project_id: &str,
        app_id: i64,
        country: &str,
        auto_refresh: bool,
    ) -> Result<Storefront> {
        let country = validate_country(country)?;
        let project = self.manager.get(project_id).await?;
        let app = find_app(&project.pool, app_id).await?;
        let created_at = Utc::now().to_rfc3339();
        sqlx::query(indoc! {r#"
            INSERT INTO app_storefronts (
                app_id,
                country,
                is_main,
                auto_refresh,
                created_at
            )
            VALUES (?, ?, 0, ?, ?)
        "#})
        .bind(app.id)
        .bind(&country)
        .bind(auto_refresh)
        .bind(&created_at)
        .execute(&project.pool)
        .await
        .context("this storefront is already configured for the app")?;
        Ok(Storefront {
            country,
            is_main: false,
            auto_refresh,
            created_at,
        })
    }

    pub async fn update_storefront(
        &self,
        project_id: &str,
        app_id: i64,
        country: &str,
        update: &UpdateStorefront,
    ) -> Result<Vec<Storefront>> {
        let country = validate_country(country)?;
        let project = self.manager.get(project_id).await?;
        let app = find_app(&project.pool, app_id).await?;
        require_storefront(&project.pool, app.id, &country).await?;
        let mut tx = project.pool.begin().await?;

        if update.is_main == Some(true) {
            sqlx::query(indoc! {r#"
                UPDATE app_storefronts
                SET is_main = 0
                WHERE app_id = ?
            "#})
            .bind(app.id)
            .execute(&mut *tx)
            .await?;
            sqlx::query(indoc! {r#"
                UPDATE app_storefronts
                SET is_main = 1, auto_refresh = 1
                WHERE app_id = ? AND country = ?
            "#})
            .bind(app.id)
            .bind(&country)
            .execute(&mut *tx)
            .await?;
        } else if update.is_main == Some(false) {
            bail!("set another storefront as main instead of unsetting the current main");
        }

        if let Some(auto_refresh) = update.auto_refresh {
            let is_main: bool = sqlx::query_scalar(indoc! {r#"
                SELECT is_main
                FROM app_storefronts
                WHERE app_id = ? AND country = ?
            "#})
            .bind(app.id)
            .bind(&country)
            .fetch_one(&mut *tx)
            .await?;
            if is_main && !auto_refresh {
                bail!("automatic refresh cannot be disabled for the main storefront");
            }
            sqlx::query(indoc! {r#"
                UPDATE app_storefronts
                SET auto_refresh = ?
                WHERE app_id = ? AND country = ?
            "#})
            .bind(auto_refresh)
            .bind(app.id)
            .bind(&country)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.storefronts(project_id, app_id).await
    }

    pub async fn delete_storefront(
        &self,
        project_id: &str,
        app_id: i64,
        country: &str,
    ) -> Result<()> {
        let country = validate_country(country)?;
        let project = self.manager.get(project_id).await?;
        let app = find_app(&project.pool, app_id).await?;
        let is_main: Option<bool> = sqlx::query_scalar(indoc! {r#"
            SELECT is_main
            FROM app_storefronts
            WHERE app_id = ? AND country = ?
        "#})
        .bind(app.id)
        .bind(&country)
        .fetch_optional(&project.pool)
        .await?;
        match is_main {
            None => bail!("storefront '{country}' is not configured for app {app_id}"),
            Some(true) => bail!("the main storefront cannot be removed"),
            Some(false) => {}
        }
        sqlx::query(indoc! {r#"
            DELETE FROM app_storefronts
            WHERE app_id = ? AND country = ?
        "#})
        .bind(app.id)
        .bind(&country)
        .execute(&project.pool)
        .await?;
        Ok(())
    }

    pub async fn refresh_app(
        &self,
        project_id: &str,
        app_id: i64,
        country: Option<&str>,
    ) -> Result<()> {
        self.refresh_app_scope(project_id, app_id, country, false, true)
            .await
    }

    pub async fn refresh_all_storefronts(&self, project_id: &str, app_id: i64) -> Result<()> {
        self.refresh_app_scope(project_id, app_id, None, true, true)
            .await
    }

    async fn refresh_app_scope(
        &self,
        project_id: &str,
        app_id: i64,
        country: Option<&str>,
        all_storefronts: bool,
        refresh_estimates: bool,
    ) -> Result<()> {
        let project = self.manager.get(project_id).await?;
        let app = find_app(&project.pool, app_id).await?;
        let countries = match country {
            Some(country) => {
                let country = validate_country(country)?;
                require_storefront(&project.pool, app.id, &country).await?;
                vec![country]
            }
            None => {
                sqlx::query_scalar::<_, String>(indoc! {r#"
                    SELECT country
                    FROM app_storefronts
                    WHERE app_id = ?
                      AND (? = 1 OR is_main = 1 OR auto_refresh = 1)
                    ORDER BY is_main DESC, country
                "#})
                .bind(app.id)
                .bind(all_storefronts)
                .fetch_all(&project.pool)
                .await?
            }
        };

        for country in &countries {
            self.refresh_storefront(&project, &app, country).await?;
        }
        if refresh_estimates {
            if let Err(error) = self
                .refresh_estimates(&project, &app, all_storefronts)
                .await
            {
                tracing::warn!(
                    project_id,
                    app_id,
                    %error,
                    "market estimate refresh failed"
                );
            }
        }
        self.refresh_popularity(&project, &app).await?;
        prune_history(&project.pool).await?;
        Ok(())
    }

    pub async fn reviews_page(
        &self,
        project_id: &str,
        app_id: i64,
        country: &str,
        page: u8,
        rating: Option<u8>,
    ) -> Result<ReviewsPage> {
        if !(1..=10).contains(&page) {
            bail!("review page must be between 1 and 10");
        }
        if rating.is_some_and(|rating| !(1..=5).contains(&rating)) {
            bail!("review rating must be between 1 and 5");
        }
        let country = validate_country(country)?;
        let project = self.manager.get(project_id).await?;
        let app = find_app(&project.pool, app_id).await?;
        require_storefront(&project.pool, app.id, &country).await?;
        let pages = if rating.is_some() {
            1..=10
        } else {
            page..=page
        };
        for source_page in pages {
            let fetch = sqlx::query(indoc! {r#"
                SELECT fetched_at, result_count
                FROM review_page_fetches
                WHERE app_id = ? AND country = ? AND page = ?
            "#})
            .bind(app.id)
            .bind(&country)
            .bind(source_page)
            .fetch_optional(&project.pool)
            .await?;
            let fetched_at = fetch
                .as_ref()
                .map(|row| row.try_get::<String, _>("fetched_at"))
                .transpose()?;
            if fetched_at.as_deref().is_none_or(is_stale) {
                self.fetch_review_page(&project, &app, &country, source_page)
                    .await?;
            }
            let result_count: i64 = sqlx::query_scalar(indoc! {r#"
                SELECT result_count
                FROM review_page_fetches
                WHERE app_id = ? AND country = ? AND page = ?
            "#})
            .bind(app.id)
            .bind(&country)
            .bind(source_page)
            .fetch_one(&project.pool)
            .await?;
            if result_count < 50 {
                break;
            }
        }

        let page_size = 50_u32;
        let offset = i64::from(page - 1) * i64::from(page_size);
        let reviews = match rating {
            Some(rating) => {
                sqlx::query_as::<_, Review>(indoc! {r#"
                    SELECT
                        review_id,
                        author,
                        rating,
                        title,
                        content,
                        version,
                        helpful_score,
                        helpful_vote_count,
                        updated_at,
                        first_seen_at,
                        last_seen_at
                    FROM reviews
                    WHERE app_id = ? AND country = ? AND rating = ?
                    ORDER BY COALESCE(updated_at, last_seen_at) DESC, review_id DESC
                    LIMIT ? OFFSET ?
                "#})
                .bind(app.id)
                .bind(&country)
                .bind(rating)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&project.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, Review>(indoc! {r#"
                    SELECT
                        review_id,
                        author,
                        rating,
                        title,
                        content,
                        version,
                        helpful_score,
                        helpful_vote_count,
                        updated_at,
                        first_seen_at,
                        last_seen_at
                    FROM reviews
                    WHERE app_id = ? AND country = ?
                    ORDER BY COALESCE(updated_at, last_seen_at) DESC, review_id DESC
                    LIMIT ? OFFSET ?
                "#})
                .bind(app.id)
                .bind(&country)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&project.pool)
                .await?
            }
        };
        let total: i64 = match rating {
            Some(rating) => {
                sqlx::query_scalar(indoc! {r#"
                    SELECT COUNT(*)
                    FROM reviews
                    WHERE app_id = ? AND country = ? AND rating = ?
                "#})
                .bind(app.id)
                .bind(&country)
                .bind(rating)
                .fetch_one(&project.pool)
                .await?
            }
            None => {
                sqlx::query_scalar(indoc! {r#"
                    SELECT COUNT(*)
                    FROM reviews
                    WHERE app_id = ? AND country = ?
                "#})
                .bind(app.id)
                .bind(&country)
                .fetch_one(&project.pool)
                .await?
            }
        };
        let fetched_at = sqlx::query_scalar(indoc! {r#"
            SELECT MAX(fetched_at)
            FROM review_page_fetches
            WHERE app_id = ? AND country = ?
        "#})
        .bind(app.id)
        .bind(&country)
        .fetch_one(&project.pool)
        .await?;
        let returned = i64::try_from(reviews.len())?;
        let summary = review_summary(&project.pool, app.id, &country).await?;
        Ok(ReviewsPage {
            country,
            page,
            page_size,
            total,
            total_all: summary.count,
            has_more: offset + returned < total,
            rating_counts: summary.rating_counts,
            fetched_at,
            reviews,
        })
    }

    pub async fn add_keyword(&self, project_id: &str, input: &AddKeyword) -> Result<KeywordEntity> {
        let identity = KeywordIdentity::new(&input.keyword, &input.country)?;
        let project = self.manager.get(project_id).await?;
        let result = sqlx::query(indoc! {r#"
            INSERT INTO keyword_queries (
                query,
                normalized_query,
                country,
                notes,
                created_at
            )
            VALUES (?, ?, ?, ?, ?)
        "#})
        .bind(identity.display())
        .bind(identity.normalized().as_str())
        .bind(identity.country())
        .bind(input.notes.trim())
        .bind(Utc::now().to_rfc3339())
        .execute(&project.pool)
        .await
        .context("this keyword and storefront are already tracked in the project")?;
        let query_id = result.last_insert_rowid();
        self.refresh_keyword_query(project_id, &project, query_id, false)
            .await?;
        keyword_entity_by_id(&project.pool, query_id).await
    }

    pub async fn update_keyword_metrics(
        &self,
        project_id: &str,
        input: &UpdateKeywordMetrics,
    ) -> Result<KeywordEntity> {
        if !input.difficulty.is_present() && !input.popularity.is_present() {
            bail!("provide difficulty, popularity, or both");
        }
        for (name, patch) in [
            ("difficulty", &input.difficulty),
            ("popularity", &input.popularity),
        ] {
            if let PatchValue::Present(Some(value)) = patch {
                validate_keyword_metric(name, *value)?;
            }
        }

        let identity = KeywordIdentity::new(&input.keyword, &input.country)?;
        let project = self.manager.get(project_id).await?;
        let current = sqlx::query_as::<_, KeywordEntityRecord>(indoc! {r#"
            SELECT
                id AS query_id,
                query,
                normalized_query,
                country,
                notes,
                difficulty,
                popularity
            FROM keyword_queries
            WHERE normalized_query = ? AND country = ?
        "#})
        .bind(identity.normalized().as_str())
        .bind(identity.country())
        .fetch_optional(&project.pool)
        .await?
        .with_context(|| {
            format!(
                "keyword '{}' is not tracked in storefront {}",
                identity.display(),
                identity.country().to_uppercase()
            )
        })?;

        let difficulty = input.difficulty.clone().apply(current.difficulty);
        let popularity = input.popularity.clone().apply(current.popularity);
        sqlx::query(indoc! {r#"
            UPDATE keyword_queries
            SET difficulty = ?, popularity = ?
            WHERE id = ?
        "#})
        .bind(difficulty)
        .bind(popularity)
        .bind(current.query_id)
        .execute(&project.pool)
        .await?;

        Ok(keyword_entity(
            current.query_id,
            current.query,
            current.normalized_query,
            current.country,
            current.notes,
            difficulty,
            popularity,
        ))
    }

    pub async fn update_keyword(
        &self,
        project_id: &str,
        query_id: i64,
        notes: &str,
    ) -> Result<KeywordEntity> {
        let project = self.manager.get(project_id).await?;
        let result = sqlx::query(indoc! {r#"
            UPDATE keyword_queries
            SET notes = ?
            WHERE id = ?
        "#})
        .bind(notes.trim())
        .bind(query_id)
        .execute(&project.pool)
        .await?;
        if result.rows_affected() == 0 {
            bail!("keyword {query_id} is not tracked in this project");
        }
        keyword_entity_by_id(&project.pool, query_id).await
    }

    pub async fn delete_keyword(&self, project_id: &str, query_id: i64) -> Result<()> {
        let project = self.manager.get(project_id).await?;
        let result = sqlx::query(indoc! {r#"
            DELETE FROM keyword_queries
            WHERE id = ?
        "#})
        .bind(query_id)
        .execute(&project.pool)
        .await?;
        if result.rows_affected() == 0 {
            bail!("keyword {query_id} is not tracked in this project");
        }
        Ok(())
    }

    pub async fn keywords(
        &self,
        project_id: &str,
        app_id: Option<i64>,
    ) -> Result<Vec<KeywordView>> {
        let project = self.manager.get(project_id).await?;
        let tracked_apple_id = match app_id {
            Some(app_id) => Some(find_app(&project.pool, app_id).await?.apple_id),
            None => None,
        };
        let records = sqlx::query_as::<_, KeywordRecord>(indoc! {r#"
            SELECT
                q.id AS query_id,
                q.query,
                q.normalized_query,
                q.country,
                q.notes,
                q.difficulty,
                q.popularity
            FROM keyword_queries q
            ORDER BY q.query COLLATE NOCASE, q.country
        "#})
        .fetch_all(&project.pool)
        .await?;
        let mut keywords = Vec::with_capacity(records.len());
        for record in records {
            keywords.push(keyword_view(&project.pool, tracked_apple_id, record).await?);
        }
        Ok(keywords)
    }

    pub async fn keyword(
        &self,
        project_id: &str,
        app_id: i64,
        query_id: i64,
    ) -> Result<KeywordView> {
        let project = self.manager.get(project_id).await?;
        let app = find_app(&project.pool, app_id).await?;
        let record = sqlx::query_as::<_, KeywordRecord>(indoc! {r#"
            SELECT
                q.id AS query_id,
                q.query,
                q.normalized_query,
                q.country,
                q.notes,
                q.difficulty,
                q.popularity
            FROM keyword_queries q
            WHERE q.id = ?
        "#})
        .bind(query_id)
        .fetch_one(&project.pool)
        .await
        .context("keyword is not tracked in this project")?;
        keyword_view(&project.pool, Some(app.apple_id), record).await
    }

    pub async fn refresh_keywords(
        &self,
        project_id: &str,
        app_id: Option<i64>,
        query_id: Option<i64>,
        force: bool,
    ) -> Result<Vec<KeywordView>> {
        let project = self.manager.get(project_id).await?;
        if let Some(app_id) = app_id {
            find_app(&project.pool, app_id).await?;
        }
        let ids = match query_id {
            Some(query_id) => {
                let exists: bool = sqlx::query_scalar(indoc! {r#"
                    SELECT EXISTS(
                        SELECT 1
                        FROM keyword_queries
                        WHERE id = ?
                    )
                "#})
                .bind(query_id)
                .fetch_one(&project.pool)
                .await?;
                if !exists {
                    bail!("keyword {query_id} is not tracked in this project");
                }
                vec![query_id]
            }
            None => {
                sqlx::query_scalar(indoc! {r#"
                    SELECT id
                    FROM keyword_queries
                "#})
                .fetch_all(&project.pool)
                .await?
            }
        };
        for query_id in ids {
            self.refresh_keyword_query(project_id, &project, query_id, force)
                .await?;
        }
        prune_history(&project.pool).await?;
        self.keywords(project_id, app_id).await
    }

    pub async fn refresh_stale(&self) -> Result<()> {
        for project in self.manager.list().await {
            let handle = self.manager.get(&project.id).await?;
            let app_ids = sqlx::query_scalar::<_, i64>(indoc! {r#"
                SELECT DISTINCT a.apple_id
                FROM apps a
                JOIN app_storefronts sf ON sf.app_id = a.id
                LEFT JOIN app_snapshots latest
                  ON latest.id = (
                      SELECT snapshots.id
                      FROM app_snapshots snapshots
                      WHERE snapshots.app_id = a.id
                        AND snapshots.country = sf.country
                      ORDER BY snapshots.fetched_at DESC
                      LIMIT 1
                  )
                WHERE (sf.is_main = 1 OR sf.auto_refresh = 1)
                  AND (
                      latest.fetched_at IS NULL
                      OR datetime(latest.fetched_at) <= datetime('now', '-24 hours')
                  )
            "#})
            .fetch_all(&handle.pool)
            .await?;
            let mut apps = Vec::with_capacity(app_ids.len());
            for app_id in app_ids {
                apps.push(find_app(&handle.pool, app_id).await?);
            }
            if let Err(error) = self.refresh_estimates_batch(&handle, &apps, false).await {
                tracing::warn!(
                    project_id = %project.id,
                    %error,
                    "batched market estimate refresh failed"
                );
            }
            for app in apps {
                if let Err(error) = self
                    .refresh_app_scope(&project.id, app.apple_id, None, false, false)
                    .await
                {
                    tracing::warn!(project_id = %project.id, app_id = app.apple_id, %error, "automatic refresh failed");
                }
            }
        }
        Ok(())
    }

    async fn refresh_storefront(
        &self,
        project: &ProjectHandle,
        app: &AppRecord,
        country: &str,
    ) -> Result<()> {
        let specifier = AppSpecifier {
            id: u64::try_from(app.apple_id)?,
            country: None,
        };
        let lookup = commands::lookup::run_without_estimates(
            &self.client,
            &LookupRequest {
                apps: vec![specifier.clone()],
                country: Some(country.to_string()),
                full: true,
            },
        )
        .await?;
        if lookup.data.as_array().is_none_or(Vec::is_empty) {
            bail!(
                "app {} is not available in the {country} storefront",
                app.apple_id
            );
        }
        store_snapshot(&project.pool, app.id, country, &lookup).await?;

        self.fetch_review_page(project, app, country, 1).await?;
        Ok(())
    }

    async fn refresh_estimates(
        &self,
        project: &ProjectHandle,
        app: &AppRecord,
        force: bool,
    ) -> Result<()> {
        self.refresh_estimates_batch(project, std::slice::from_ref(app), force)
            .await
    }

    async fn refresh_estimates_batch(
        &self,
        project: &ProjectHandle,
        apps: &[AppRecord],
        force: bool,
    ) -> Result<()> {
        let mut pending = Vec::with_capacity(apps.len());
        for app in apps {
            let latest: Option<String> = sqlx::query_scalar(indoc! {r#"
                SELECT fetched_at
                FROM app_estimate_snapshots
                WHERE app_id = ?
                ORDER BY fetched_at DESC
                LIMIT 1
            "#})
            .bind(app.id)
            .fetch_optional(&project.pool)
            .await?;
            if force || latest.as_deref().is_none_or(is_stale) {
                pending.push((app.id, u64::try_from(app.apple_id)?));
            }
        }
        if pending.is_empty() {
            return Ok(());
        }

        let app_ids = pending
            .iter()
            .map(|(_, apple_id)| *apple_id)
            .collect::<Vec<_>>();
        let mut estimates = market_estimates::fetch_app_estimates(&self.client, &app_ids)
            .await?
            .into_iter()
            .map(|estimate| (estimate.app_id, estimate))
            .collect::<HashMap<_, _>>();
        for (internal_id, apple_id) in pending {
            let estimate = estimates.remove(&apple_id).with_context(|| {
                format!("market data provider returned no estimates for app {apple_id}")
            })?;
            store_app_estimates(&project.pool, internal_id, &estimate).await?;
        }
        Ok(())
    }

    async fn refresh_popularity(&self, project: &ProjectHandle, app: &AppRecord) -> Result<()> {
        let envelope = commands::popularity::run(
            &self.client,
            &PopularityRequest {
                app: AppSpecifier {
                    id: u64::try_from(app.apple_id)?,
                    country: None,
                },
                group: PopularityGroup::Tier2,
                countries: None,
            },
        )
        .await?;
        let mut tx = project.pool.begin().await?;
        let requested = envelope
            .meta
            .parameters
            .get("countries")
            .cloned()
            .unwrap_or_else(|| json!([]));
        let run = sqlx::query(indoc! {r#"
            INSERT INTO popularity_runs (
                app_id,
                fetched_at,
                group_name,
                requested_countries_json
            )
            VALUES (?, ?, ?, ?)
        "#})
        .bind(app.id)
        .bind(&envelope.meta.retrieved_at)
        .bind("tier2")
        .bind(serde_json::to_string(&requested)?)
        .execute(&mut *tx)
        .await?;
        let run_id = run.last_insert_rowid();
        if let Some(records) = envelope.data.as_array() {
            for record in records {
                sqlx::query(indoc! {r#"
                    INSERT INTO popularity_observations (
                        run_id,
                        country,
                        available,
                        name,
                        rating,
                        rating_count
                    )
                    VALUES (?, ?, ?, ?, ?, ?)
                "#})
                .bind(run_id)
                .bind(required_string(record, "country")?)
                .bind(
                    record
                        .get("available")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                )
                .bind(optional_string(record, "name"))
                .bind(record.get("rating").and_then(Value::as_f64))
                .bind(record.get("rating_count").and_then(Value::as_i64))
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    async fn fetch_review_page(
        &self,
        project: &ProjectHandle,
        app: &AppRecord,
        country: &str,
        page: u8,
    ) -> Result<()> {
        let envelope = commands::reviews::run(
            &self.client,
            &ReviewsRequest {
                app: AppSpecifier {
                    id: u64::try_from(app.apple_id)?,
                    country: None,
                },
                country: Some(country.to_string()),
                page: Some(page),
                pages: None,
                all: false,
            },
        )
        .await?;
        let records = envelope.data.as_array().map(Vec::as_slice).unwrap_or(&[]);
        let mut tx = project.pool.begin().await?;
        for record in records {
            sqlx::query(indoc! {r#"
                INSERT INTO reviews (
                    app_id,
                    country,
                    review_id,
                    author,
                    rating,
                    title,
                    content,
                    version,
                    helpful_score,
                    helpful_vote_count,
                    updated_at,
                    first_seen_at,
                    last_seen_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT (app_id, country, review_id) DO UPDATE SET
                    author = excluded.author,
                    rating = excluded.rating,
                    title = excluded.title,
                    content = excluded.content,
                    version = excluded.version,
                    helpful_score = excluded.helpful_score,
                    helpful_vote_count = excluded.helpful_vote_count,
                    updated_at = excluded.updated_at,
                    last_seen_at = excluded.last_seen_at
            "#})
            .bind(app.id)
            .bind(country)
            .bind(required_i64(record, "id")?)
            .bind(optional_string(record, "author"))
            .bind(required_i64(record, "rating")?)
            .bind(optional_string(record, "title"))
            .bind(required_string(record, "content")?)
            .bind(optional_string(record, "version"))
            .bind(record.get("helpful_score").and_then(Value::as_i64))
            .bind(record.get("helpful_vote_count").and_then(Value::as_i64))
            .bind(optional_string(record, "updated_at"))
            .bind(&envelope.meta.retrieved_at)
            .bind(&envelope.meta.retrieved_at)
            .execute(&mut *tx)
            .await?;
        }
        sqlx::query(indoc! {r#"
            INSERT INTO review_page_fetches (
                app_id,
                country,
                page,
                fetched_at,
                result_count
            )
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT (app_id, country, page) DO UPDATE SET
                fetched_at = excluded.fetched_at,
                result_count = excluded.result_count
        "#})
        .bind(app.id)
        .bind(country)
        .bind(page)
        .bind(&envelope.meta.retrieved_at)
        .bind(i64::try_from(records.len())?)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn refresh_keyword_query(
        &self,
        project_id: &str,
        project: &ProjectHandle,
        query_id: i64,
        force: bool,
    ) -> Result<()> {
        let lock_key = format!("{project_id}:{query_id}");
        let lock = {
            let mut locks = self.keyword_locks.lock().await;
            locks
                .entry(lock_key)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let _guard = lock.lock().await;
        let latest: Option<String> = sqlx::query_scalar(indoc! {r#"
            SELECT fetched_at
            FROM keyword_query_runs
            WHERE query_id = ?
            ORDER BY fetched_at DESC
            LIMIT 1
        "#})
        .bind(query_id)
        .fetch_optional(&project.pool)
        .await?;
        if !force && latest.as_deref().is_some_and(|value| !is_stale(value)) {
            let has_legacy_result_shape = sqlx::query_scalar::<_, bool>(indoc! {r#"
                    SELECT
                        EXISTS (
                            SELECT 1
                            FROM keyword_results results
                            WHERE results.run_id = runs.id
                        )
                        AND NOT EXISTS (
                            SELECT 1
                            FROM keyword_results results
                            WHERE results.run_id = runs.id
                              AND (
                                  results.released_at IS NOT NULL
                                  OR results.version_released_at IS NOT NULL
                                  OR results.rating IS NOT NULL
                                  OR results.rating_count IS NOT NULL
                              )
                        )
                    FROM keyword_query_runs runs
                    WHERE runs.query_id = ?
                    ORDER BY runs.fetched_at DESC
                    LIMIT 1
                "#})
            .bind(query_id)
            .fetch_optional(&project.pool)
            .await?
            .unwrap_or(false);
            if !has_legacy_result_shape {
                return Ok(());
            }
        }
        let row = sqlx::query(indoc! {r#"
            SELECT query, country
            FROM keyword_queries
            WHERE id = ?
        "#})
        .bind(query_id)
        .fetch_one(&project.pool)
        .await?;
        let query: String = row.try_get("query")?;
        let country: String = row.try_get("country")?;
        let envelope = commands::search::run(
            &self.client,
            &SearchRequest {
                term: query,
                country: Some(country),
                limit: 200,
                local_limit: None,
            },
        )
        .await?;
        let records = envelope.data.as_array().map(Vec::as_slice).unwrap_or(&[]);
        let mut tx = project.pool.begin().await?;
        let run = sqlx::query(indoc! {r#"
            INSERT INTO keyword_query_runs (query_id, fetched_at, result_count)
            VALUES (?, ?, ?)
        "#})
        .bind(query_id)
        .bind(&envelope.meta.retrieved_at)
        .bind(i64::try_from(records.len())?)
        .execute(&mut *tx)
        .await?;
        let run_id = run.last_insert_rowid();
        for record in records {
            sqlx::query(indoc! {r#"
                INSERT INTO keyword_results (
                    run_id,
                    position,
                    apple_id,
                    name,
                    icon_url,
                    developer_name,
                    released_at,
                    version_released_at,
                    rating,
                    rating_count
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#})
            .bind(run_id)
            .bind(required_i64(record, "position")?)
            .bind(required_i64(record, "app_id")?)
            .bind(required_string(record, "name")?)
            .bind(optional_string(record, "icon_url"))
            .bind(optional_string(record, "developer_name"))
            .bind(optional_string(record, "released_at"))
            .bind(optional_string(record, "version_released_at"))
            .bind(record.get("rating").and_then(Value::as_f64))
            .bind(record.get("rating_count").and_then(Value::as_i64))
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

async fn find_app(pool: &SqlitePool, apple_id: i64) -> Result<AppRecord> {
    sqlx::query_as::<_, AppRecord>(indoc! {r#"
        SELECT id, apple_id, created_at
        FROM apps
        WHERE apple_id = ?
    "#})
    .bind(apple_id)
    .fetch_optional(pool)
    .await?
    .with_context(|| format!("app {apple_id} is not tracked in this project"))
}

async fn require_storefront(pool: &SqlitePool, app_id: i64, country: &str) -> Result<()> {
    let exists: bool = sqlx::query_scalar(indoc! {r#"
        SELECT EXISTS(
            SELECT 1
            FROM app_storefronts
            WHERE app_id = ? AND country = ?
        )
    "#})
    .bind(app_id)
    .bind(country)
    .fetch_one(pool)
    .await?;
    if !exists {
        bail!("storefront '{country}' is not configured for this app");
    }
    Ok(())
}

async fn store_snapshot(
    pool: &SqlitePool,
    app_id: i64,
    country: &str,
    envelope: &Envelope,
) -> Result<()> {
    sqlx::query(indoc! {r#"
        INSERT INTO app_snapshots (app_id, country, fetched_at, payload_json)
        VALUES (?, ?, ?, ?)
    "#})
    .bind(app_id)
    .bind(country)
    .bind(&envelope.meta.retrieved_at)
    .bind(serde_json::to_string(envelope)?)
    .execute(pool)
    .await?;
    Ok(())
}

async fn store_app_estimates(
    pool: &SqlitePool,
    app_id: i64,
    estimate: &market_estimates::AppEstimates,
) -> Result<()> {
    let downloads = estimate.humanized_worldwide_last_month_downloads.as_ref();
    let revenue = estimate.humanized_worldwide_last_month_revenue.as_ref();
    let downloads_value = downloads
        .map(|metric| i64::try_from(metric.downloads))
        .transpose()?;
    let downloads_rounded = downloads
        .map(|metric| i64::try_from(metric.downloads_rounded))
        .transpose()?;
    let revenue_value = revenue
        .map(|metric| i64::try_from(metric.revenue))
        .transpose()?;
    let revenue_rounded = revenue
        .map(|metric| i64::try_from(metric.revenue_rounded))
        .transpose()?;
    sqlx::query(indoc! {r#"
        INSERT INTO app_estimate_snapshots (
            app_id,
            fetched_at,
            downloads_value,
            downloads_rounded,
            downloads_prefix,
            downloads_display,
            downloads_units,
            revenue_value,
            revenue_rounded,
            revenue_prefix,
            revenue_display,
            revenue_units,
            revenue_currency
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#})
    .bind(app_id)
    .bind(Utc::now().to_rfc3339())
    .bind(downloads_value)
    .bind(downloads_rounded)
    .bind(downloads.and_then(|metric| metric.prefix.as_deref()))
    .bind(downloads.map(|metric| metric.string.as_str()))
    .bind(downloads.map(|metric| metric.units.as_str()))
    .bind(revenue_value)
    .bind(revenue_rounded)
    .bind(revenue.and_then(|metric| metric.prefix.as_deref()))
    .bind(revenue.map(|metric| metric.string.as_str()))
    .bind(revenue.map(|metric| metric.units.as_str()))
    .bind(revenue.map(|_| "USD"))
    .execute(pool)
    .await?;
    Ok(())
}

async fn latest_app_estimates(pool: &SqlitePool, app_id: i64) -> Result<Option<AppEstimatesView>> {
    let row = sqlx::query_as::<_, EstimateSnapshotRow>(indoc! {r#"
        SELECT
            fetched_at,
            downloads_value,
            downloads_rounded,
            downloads_prefix,
            downloads_display,
            downloads_units,
            revenue_value,
            revenue_rounded,
            revenue_prefix,
            revenue_display,
            revenue_units,
            revenue_currency
        FROM app_estimate_snapshots
        WHERE app_id = ?
        ORDER BY fetched_at DESC
        LIMIT 1
    "#})
    .bind(app_id)
    .fetch_optional(pool)
    .await?;
    row.map(app_estimates_view).transpose()
}

fn app_estimates_view(row: EstimateSnapshotRow) -> Result<AppEstimatesView> {
    Ok(AppEstimatesView {
        fetched_at: row.fetched_at,
        source: "Market estimate".to_string(),
        scope: "worldwide".to_string(),
        period: "last_month".to_string(),
        downloads: app_estimate_metric(
            row.downloads_value,
            row.downloads_rounded,
            row.downloads_prefix,
            row.downloads_display,
            row.downloads_units,
        )?,
        revenue: app_estimate_metric(
            row.revenue_value,
            row.revenue_rounded,
            row.revenue_prefix,
            row.revenue_display,
            row.revenue_units,
        )?,
        revenue_currency: row.revenue_currency,
    })
}

fn app_estimate_metric(
    value: Option<i64>,
    rounded_value: Option<i64>,
    prefix: Option<String>,
    display: Option<String>,
    units: Option<String>,
) -> Result<Option<AppEstimateMetric>> {
    match value {
        None => Ok(None),
        Some(value) => Ok(Some(AppEstimateMetric {
            value,
            rounded_value: rounded_value.context("estimate is missing its rounded value")?,
            prefix,
            display: display.context("estimate is missing its display string")?,
            units: units.context("estimate is missing its units")?,
        })),
    }
}

async fn latest_payload(
    pool: &SqlitePool,
    query: &str,
    app_id: i64,
    country: &str,
    resource: Option<&str>,
) -> Result<Option<(Value, String)>> {
    let mut query = sqlx::query(query).bind(app_id).bind(country);
    if let Some(resource) = resource {
        query = query.bind(resource);
    }
    let row = query.fetch_optional(pool).await?;
    row.map(|row| {
        let payload: String = row.try_get("payload_json")?;
        let fetched_at: String = row.try_get("fetched_at")?;
        Ok((serde_json::from_str(&payload)?, fetched_at))
    })
    .transpose()
}

async fn latest_popularity(pool: &SqlitePool, app_id: i64) -> Result<Option<PopularityView>> {
    let row = sqlx::query(indoc! {r#"
        SELECT id, fetched_at, group_name
        FROM popularity_runs
        WHERE app_id = ?
        ORDER BY fetched_at DESC
        LIMIT 1
    "#})
    .bind(app_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let run_id: i64 = row.try_get("id")?;
    let countries = sqlx::query_as::<_, PopularityCountry>(indoc! {r#"
        SELECT
            country,
            available,
            name,
            rating,
            rating_count
        FROM popularity_observations
        WHERE run_id = ?
        ORDER BY rating_count DESC NULLS LAST, country
    "#})
    .bind(run_id)
    .fetch_all(pool)
    .await?;
    Ok(Some(PopularityView {
        fetched_at: row.try_get("fetched_at")?,
        group: row.try_get("group_name")?,
        countries,
    }))
}

async fn popularity_history(pool: &SqlitePool, app_id: i64) -> Result<Vec<HistorySnapshot>> {
    let runs = sqlx::query(indoc! {r#"
        SELECT id, fetched_at, group_name
        FROM popularity_runs
        WHERE app_id = ?
          AND datetime(fetched_at) >= datetime('now', '-30 days')
        ORDER BY fetched_at DESC
    "#})
    .bind(app_id)
    .fetch_all(pool)
    .await?;
    let mut history = Vec::with_capacity(runs.len());
    for run in runs {
        let run_id: i64 = run.try_get("id")?;
        let countries = sqlx::query_as::<_, PopularityCountry>(indoc! {r#"
                SELECT
                    country,
                    available,
                    name,
                    rating,
                    rating_count
                FROM popularity_observations
                WHERE run_id = ?
                ORDER BY rating_count DESC NULLS LAST, country
            "#})
        .bind(run_id)
        .fetch_all(pool)
        .await?;
        history.push(HistorySnapshot {
            resource: "popularity".to_string(),
            country: None,
            fetched_at: run.try_get("fetched_at")?,
            payload: json!({
                "group": run.try_get::<Option<String>, _>("group_name")?,
                "countries": countries,
            }),
        });
    }
    Ok(history)
}

async fn estimate_history(pool: &SqlitePool, app_id: i64) -> Result<Vec<HistorySnapshot>> {
    let rows = sqlx::query_as::<_, EstimateSnapshotRow>(indoc! {r#"
        SELECT
            fetched_at,
            downloads_value,
            downloads_rounded,
            downloads_prefix,
            downloads_display,
            downloads_units,
            revenue_value,
            revenue_rounded,
            revenue_prefix,
            revenue_display,
            revenue_units,
            revenue_currency
        FROM app_estimate_snapshots
        WHERE app_id = ?
          AND datetime(fetched_at) >= datetime('now', '-30 days')
        ORDER BY fetched_at DESC
    "#})
    .bind(app_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let view = app_estimates_view(row)?;
            Ok(HistorySnapshot {
                resource: "estimates".to_string(),
                country: None,
                fetched_at: view.fetched_at.clone(),
                payload: serde_json::to_value(view)?,
            })
        })
        .collect()
}

async fn review_summary(pool: &SqlitePool, app_id: i64, country: &str) -> Result<ReviewSummary> {
    let row = sqlx::query(indoc! {r#"
        SELECT
            COUNT(reviews.review_id) AS count,
            AVG(reviews.rating) AS average_rating,
            COALESCE(SUM(CASE WHEN reviews.rating = 1 THEN 1 ELSE 0 END), 0)
                AS rating_1,
            COALESCE(SUM(CASE WHEN reviews.rating = 2 THEN 1 ELSE 0 END), 0)
                AS rating_2,
            COALESCE(SUM(CASE WHEN reviews.rating = 3 THEN 1 ELSE 0 END), 0)
                AS rating_3,
            COALESCE(SUM(CASE WHEN reviews.rating = 4 THEN 1 ELSE 0 END), 0)
                AS rating_4,
            COALESCE(SUM(CASE WHEN reviews.rating = 5 THEN 1 ELSE 0 END), 0)
                AS rating_5,
            (
                SELECT fetched_at
                FROM review_page_fetches
                WHERE app_id = ? AND country = ? AND page = 1
            ) AS page_one_updated_at
        FROM reviews
        WHERE reviews.app_id = ? AND reviews.country = ?
    "#})
    .bind(app_id)
    .bind(country)
    .bind(app_id)
    .bind(country)
    .fetch_one(pool)
    .await?;
    Ok(ReviewSummary {
        count: row.try_get("count")?,
        average_rating: row.try_get("average_rating")?,
        page_one_updated_at: row.try_get("page_one_updated_at")?,
        rating_counts: [
            row.try_get("rating_1")?,
            row.try_get("rating_2")?,
            row.try_get("rating_3")?,
            row.try_get("rating_4")?,
            row.try_get("rating_5")?,
        ],
    })
}

async fn keyword_view(
    pool: &SqlitePool,
    tracked_apple_id: Option<i64>,
    record: KeywordRecord,
) -> Result<KeywordView> {
    let runs = sqlx::query(indoc! {r#"
        SELECT
            runs.id,
            runs.fetched_at,
            results.position
        FROM keyword_query_runs runs
        LEFT JOIN keyword_results results
          ON results.run_id = runs.id
         AND results.apple_id = ?
        WHERE runs.query_id = ?
        ORDER BY runs.fetched_at DESC
        LIMIT 2
    "#})
    .bind(tracked_apple_id)
    .bind(record.query_id)
    .fetch_all(pool)
    .await?;
    let last_updated = runs
        .first()
        .map(|row| row.try_get::<String, _>("fetched_at"))
        .transpose()?;
    let position = runs
        .first()
        .map(|row| row.try_get::<Option<i64>, _>("position"))
        .transpose()?
        .flatten();
    let previous_position = runs
        .get(1)
        .map(|row| row.try_get::<Option<i64>, _>("position"))
        .transpose()?
        .flatten();
    let trend = match tracked_apple_id {
        Some(tracked_apple_id) => {
            sqlx::query_as::<_, KeywordTrendPoint>(indoc! {r#"
                SELECT
                    runs.fetched_at,
                    results.position
                FROM keyword_query_runs runs
                LEFT JOIN keyword_results results
                  ON results.run_id = runs.id
                 AND results.apple_id = ?
                WHERE runs.query_id = ?
                  AND datetime(runs.fetched_at) >= datetime('now', '-30 days')
                ORDER BY runs.fetched_at
            "#})
            .bind(tracked_apple_id)
            .bind(record.query_id)
            .fetch_all(pool)
            .await?
        }
        None => Vec::new(),
    };
    let latest_run_id: Option<i64> = sqlx::query_scalar(indoc! {r#"
        SELECT id
        FROM keyword_query_runs
        WHERE query_id = ?
        ORDER BY fetched_at DESC
        LIMIT 1
    "#})
    .bind(record.query_id)
    .fetch_optional(pool)
    .await?;
    let apps_in_ranking = match latest_run_id {
        Some(run_id) => {
            sqlx::query_as::<_, RankedApp>(indoc! {r#"
                SELECT
                    position,
                    apple_id,
                    name,
                    icon_url,
                    developer_name,
                    released_at,
                    version_released_at,
                    rating,
                    rating_count
                FROM keyword_results
                WHERE run_id = ?
                ORDER BY position
            "#})
            .bind(run_id)
            .fetch_all(pool)
            .await?
        }
        None => Vec::new(),
    };
    Ok(KeywordView {
        entity: keyword_entity(
            record.query_id,
            record.query,
            record.normalized_query,
            record.country,
            record.notes,
            record.difficulty,
            record.popularity,
        ),
        last_updated,
        position,
        previous_position,
        trend,
        apps_in_ranking,
    })
}

async fn prune_history(pool: &SqlitePool) -> Result<()> {
    let mut tx = pool.begin().await?;
    for query in [
        indoc! {r#"
            DELETE FROM app_snapshots
            WHERE datetime(fetched_at) < datetime('now', '-30 days')
        "#},
        indoc! {r#"
            DELETE FROM app_resource_snapshots
            WHERE datetime(fetched_at) < datetime('now', '-30 days')
        "#},
        indoc! {r#"
            DELETE FROM app_estimate_snapshots
            WHERE datetime(fetched_at) < datetime('now', '-30 days')
        "#},
        indoc! {r#"
            DELETE FROM popularity_runs
            WHERE datetime(fetched_at) < datetime('now', '-30 days')
        "#},
        indoc! {r#"
            DELETE FROM keyword_query_runs
            WHERE datetime(fetched_at) < datetime('now', '-30 days')
        "#},
        indoc! {r#"
            DELETE FROM reviews
            WHERE datetime(last_seen_at) < datetime('now', '-30 days')
        "#},
    ] {
        sqlx::query(query).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn keyword_entity_by_id(pool: &SqlitePool, query_id: i64) -> Result<KeywordEntity> {
    let record = sqlx::query_as::<_, KeywordEntityRecord>(indoc! {r#"
        SELECT
            id AS query_id,
            query,
            normalized_query,
            country,
            notes,
            difficulty,
            popularity
        FROM keyword_queries
        WHERE id = ?
    "#})
    .bind(query_id)
    .fetch_optional(pool)
    .await?
    .context("keyword is not tracked in this project")?;
    Ok(keyword_entity(
        record.query_id,
        record.query,
        record.normalized_query,
        record.country,
        record.notes,
        record.difficulty,
        record.popularity,
    ))
}

fn keyword_entity(
    query_id: i64,
    keyword: String,
    normalized_keyword: String,
    country: String,
    notes: String,
    difficulty: Option<f64>,
    popularity: Option<f64>,
) -> KeywordEntity {
    KeywordEntity {
        query_id,
        keyword,
        normalized_keyword,
        country,
        notes,
        difficulty,
        popularity,
    }
}

fn validate_keyword_metric(name: &str, value: f64) -> Result<()> {
    if !value.is_finite() || !(0.0..=100.0).contains(&value) {
        bail!("{name} must be a number from 0 to 100");
    }
    Ok(())
}

fn checked_app_id(app_id: u64) -> Result<i64> {
    if app_id == 0 {
        bail!("app ID must be positive");
    }
    i64::try_from(app_id).context("app ID is too large")
}

fn is_stale(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|date| SystemTime::from(date.with_timezone(&Utc)))
        .and_then(|then| SystemTime::now().duration_since(then).ok())
        .is_none_or(|age| age >= FRESH_FOR)
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("App Store response is missing '{field}'"))
}

fn optional_string(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(Value::as_str).map(str::to_string)
}

fn required_i64(value: &Value, field: &str) -> Result<i64> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .with_context(|| format!("App Store response is missing '{field}'"))
}

fn string_at(value: Option<&Value>, pointer: &str) -> Option<String> {
    value?
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_string)
}

#[allow(dead_code)]
async fn _transaction_marker(_: &mut Transaction<'_, Sqlite>) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn canonical_metric_updates_are_shared_and_support_partial_clears() {
        let temporary = tempfile::tempdir().unwrap();
        let manager = ProjectManager::open(Some(temporary.path().to_path_buf()))
            .await
            .unwrap();
        let project = manager.list().await.remove(0);
        let handle = manager.get(&project.id).await.unwrap();
        let now = Utc::now().to_rfc3339();

        for app_id in [1001_i64, 1002_i64] {
            sqlx::query("INSERT INTO apps (apple_id, created_at) VALUES (?, ?)")
                .bind(app_id)
                .bind(&now)
                .execute(&handle.pool)
                .await
                .unwrap();
        }
        sqlx::query(indoc! {r#"
            INSERT INTO keyword_queries (query, normalized_query, country, notes, created_at)
            VALUES ('Music Discovery', 'music discovery', 'us', 'Project note', ?)
        "#})
        .bind(&now)
        .execute(&handle.pool)
        .await
        .unwrap();
        let run = sqlx::query(indoc! {r#"
            INSERT INTO keyword_query_runs (query_id, fetched_at, result_count)
            VALUES (1, ?, 2)
        "#})
        .bind(&now)
        .execute(&handle.pool)
        .await
        .unwrap();
        let run_id = run.last_insert_rowid();
        for (position, apple_id) in [(3_i64, 1001_i64), (9_i64, 1002_i64)] {
            sqlx::query(indoc! {r#"
                INSERT INTO keyword_results (run_id, position, apple_id, name)
                VALUES (?, ?, ?, ?)
            "#})
            .bind(run_id)
            .bind(position)
            .bind(apple_id)
            .bind(format!("App {apple_id}"))
            .execute(&handle.pool)
            .await
            .unwrap();
        }

        let service = AppService::new(manager, ClientConfig::default()).unwrap();
        let update: UpdateKeywordMetrics = serde_json::from_value(json!({
            "keyword": "  MUSIC\n DISCOVERY ",
            "country": "US",
            "difficulty": 64,
            "popularity": 82.5
        }))
        .unwrap();
        let updated = service
            .update_keyword_metrics(&project.id, &update)
            .await
            .unwrap();
        assert_eq!(updated.normalized_keyword, "music discovery");
        assert_eq!(updated.difficulty, Some(64.0));
        assert_eq!(updated.popularity, Some(82.5));

        for (app_id, expected_position) in [(1001_i64, 3_i64), (1002_i64, 9_i64)] {
            let keywords = service.keywords(&project.id, Some(app_id)).await.unwrap();
            assert_eq!(keywords.len(), 1);
            assert_eq!(keywords[0].position, Some(expected_position));
            assert_eq!(keywords[0].entity.notes, "Project note");
            assert_eq!(keywords[0].entity.difficulty, Some(64.0));
            assert_eq!(keywords[0].entity.popularity, Some(82.5));
            let response = serde_json::to_value(&keywords[0]).unwrap();
            assert_eq!(response["normalized_keyword"], "music discovery");
            assert_eq!(response["notes"], "Project note");
            assert_eq!(response["difficulty"], 64.0);
            assert!(response.get("entity").is_none());
        }

        let project_keywords = service.keywords(&project.id, None).await.unwrap();
        assert_eq!(project_keywords.len(), 1);
        assert_eq!(project_keywords[0].position, None);
        assert!(project_keywords[0].trend.is_empty());
        assert_eq!(project_keywords[0].apps_in_ranking.len(), 2);

        let clear_difficulty: UpdateKeywordMetrics = serde_json::from_value(json!({
            "keyword": "music discovery",
            "country": "us",
            "difficulty": null
        }))
        .unwrap();
        let cleared = service
            .update_keyword_metrics(&project.id, &clear_difficulty)
            .await
            .unwrap();
        assert_eq!(cleared.difficulty, None);
        assert_eq!(cleared.popularity, Some(82.5));

        let invalid: UpdateKeywordMetrics = serde_json::from_value(json!({
            "keyword": "music discovery",
            "country": "us",
            "popularity": 101
        }))
        .unwrap();
        assert!(service
            .update_keyword_metrics(&project.id, &invalid)
            .await
            .is_err());

        service.delete_app(&project.id, 1001).await.unwrap();
        assert_eq!(
            service
                .keywords(&project.id, Some(1002))
                .await
                .unwrap()
                .len(),
            1,
            "deleting an app must not delete project keywords"
        );
    }
}
