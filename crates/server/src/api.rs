use appstore_api::{
    commands,
    requests::{
        ChartRequest, EstimatesRequest, ListResource, LookupRequest, PopularityRequest,
        ReviewsRequest, SearchRequest,
    },
};
use asapi_app::{
    AddApp, AddKeyword, AddStorefront, AppService, CreateProject, RefreshApp, RefreshKeyword,
    RenameProject, UpdateKeyword, UpdateStorefront,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    error::{ApiError, ApiResult},
    openapi,
};

pub fn router(service: AppService) -> Router {
    let v1 = Router::new()
        .route("/projects", get(list_projects).post(create_project))
        .route(
            "/projects/{project_id}",
            get(get_project)
                .patch(rename_project)
                .delete(delete_project),
        )
        .route("/projects/{project_id}/apps", get(list_apps).post(add_app))
        .route(
            "/projects/{project_id}/apps/{app_id}",
            get(get_app).delete(delete_app),
        )
        .route(
            "/projects/{project_id}/apps/{app_id}/refresh",
            post(refresh_app),
        )
        .route(
            "/projects/{project_id}/apps/{app_id}/history",
            get(app_history),
        )
        .route(
            "/projects/{project_id}/apps/{app_id}/storefronts",
            get(list_storefronts).post(add_storefront),
        )
        .route(
            "/projects/{project_id}/apps/{app_id}/storefronts/{country}",
            patch(update_storefront).delete(delete_storefront),
        )
        .route(
            "/projects/{project_id}/apps/{app_id}/reviews",
            get(list_reviews),
        )
        .route(
            "/projects/{project_id}/apps/{app_id}/keywords",
            get(list_keywords).post(add_keyword),
        )
        .route(
            "/projects/{project_id}/apps/{app_id}/keywords/refresh",
            post(refresh_keywords),
        )
        .route(
            "/projects/{project_id}/apps/{app_id}/keywords/{query_id}",
            patch(update_keyword).delete(delete_keyword),
        )
        .route("/query/search", post(raw_search))
        .route("/query/lookup", post(raw_lookup))
        .route("/query/estimates", post(raw_estimates))
        .route("/query/popularity", post(raw_popularity))
        .route("/query/reviews", post(raw_reviews))
        .route("/query/chart", post(raw_chart))
        .route("/query/list/{resource}", get(raw_list))
        .with_state(service);

    Router::new()
        .route("/api/health", get(health))
        .route("/api/openapi.json", get(openapi_document))
        .nest("/api/v1", v1)
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}))
}

async fn openapi_document() -> Json<Value> {
    Json(openapi::document())
}

async fn list_projects(State(service): State<AppService>) -> Json<Value> {
    Json(json!({"data": service.manager().list().await}))
}

async fn get_project(
    State(service): State<AppService>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<Value>> {
    let handle = service.manager().get(&project_id).await?;
    Ok(Json(json!({"data": handle.project})))
}

async fn create_project(
    State(service): State<AppService>,
    Json(input): Json<CreateProject>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let project = service.manager().create(&input.name).await?;
    Ok((StatusCode::CREATED, Json(json!({"data": project}))))
}

async fn rename_project(
    State(service): State<AppService>,
    Path(project_id): Path<String>,
    Json(input): Json<RenameProject>,
) -> ApiResult<Json<Value>> {
    let project = service.manager().rename(&project_id, &input.name).await?;
    Ok(Json(json!({"data": project})))
}

async fn delete_project(
    State(service): State<AppService>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<Value>> {
    let project = service.manager().delete(&project_id).await?;
    Ok(Json(json!({"data": project})))
}

async fn list_apps(
    State(service): State<AppService>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(json!({"data": service.list_apps(&project_id).await?})))
}

async fn add_app(
    State(service): State<AppService>,
    Path(project_id): Path<String>,
    Json(input): Json<AddApp>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let app = service
        .add_app(&project_id, input.app_id, &input.country)
        .await?;
    Ok((StatusCode::CREATED, Json(json!({"data": app}))))
}

#[derive(Deserialize)]
struct CountryQuery {
    country: Option<String>,
}

async fn get_app(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
    Query(query): Query<CountryQuery>,
) -> ApiResult<Json<Value>> {
    let app = service
        .app_view(&project_id, app_id, query.country.as_deref())
        .await?;
    Ok(Json(json!({"data": app})))
}

async fn delete_app(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
) -> ApiResult<StatusCode> {
    service.delete_app(&project_id, app_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn refresh_app(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
    Json(input): Json<RefreshApp>,
) -> ApiResult<Json<Value>> {
    if input.all {
        service.refresh_all_storefronts(&project_id, app_id).await?;
    } else {
        service
            .refresh_app(&project_id, app_id, input.country.as_deref())
            .await?;
    }
    let app = service
        .app_view(&project_id, app_id, input.country.as_deref())
        .await?;
    Ok(Json(json!({"data": app})))
}

#[derive(Deserialize)]
struct HistoryQuery {
    country: Option<String>,
    #[serde(default = "default_history_resource")]
    resource: String,
}

fn default_history_resource() -> String {
    "details".to_string()
}

async fn app_history(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
    Query(query): Query<HistoryQuery>,
) -> ApiResult<Json<Value>> {
    let history = service
        .app_history(
            &project_id,
            app_id,
            query.country.as_deref(),
            &query.resource,
        )
        .await?;
    Ok(Json(json!({"data": history})))
}

async fn list_storefronts(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        json!({"data": service.storefronts(&project_id, app_id).await?}),
    ))
}

async fn add_storefront(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
    Json(input): Json<AddStorefront>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let storefront = service
        .add_storefront(&project_id, app_id, &input.country, input.auto_refresh)
        .await?;
    Ok((StatusCode::CREATED, Json(json!({"data": storefront}))))
}

async fn update_storefront(
    State(service): State<AppService>,
    Path((project_id, app_id, country)): Path<(String, i64, String)>,
    Json(input): Json<UpdateStorefront>,
) -> ApiResult<Json<Value>> {
    let storefronts = service
        .update_storefront(&project_id, app_id, &country, &input)
        .await?;
    Ok(Json(json!({"data": storefronts})))
}

async fn delete_storefront(
    State(service): State<AppService>,
    Path((project_id, app_id, country)): Path<(String, i64, String)>,
) -> ApiResult<StatusCode> {
    service
        .delete_storefront(&project_id, app_id, &country)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct ReviewQuery {
    country: String,
    #[serde(default = "default_page")]
    page: u8,
    rating: Option<u8>,
}

fn default_page() -> u8 {
    1
}

async fn list_reviews(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
    Query(query): Query<ReviewQuery>,
) -> ApiResult<Json<Value>> {
    let page = service
        .reviews_page(
            &project_id,
            app_id,
            &query.country,
            query.page,
            query.rating,
        )
        .await?;
    Ok(Json(json!({"data": page})))
}

async fn list_keywords(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        json!({"data": service.keywords(&project_id, app_id).await?}),
    ))
}

async fn add_keyword(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
    Json(input): Json<AddKeyword>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let keyword = service.add_keyword(&project_id, app_id, &input).await?;
    Ok((StatusCode::CREATED, Json(json!({"data": keyword}))))
}

async fn update_keyword(
    State(service): State<AppService>,
    Path((project_id, app_id, query_id)): Path<(String, i64, i64)>,
    Json(input): Json<UpdateKeyword>,
) -> ApiResult<Json<Value>> {
    let keyword = service
        .update_keyword(&project_id, app_id, query_id, &input.notes)
        .await?;
    Ok(Json(json!({"data": keyword})))
}

async fn delete_keyword(
    State(service): State<AppService>,
    Path((project_id, app_id, query_id)): Path<(String, i64, i64)>,
) -> ApiResult<StatusCode> {
    service
        .delete_keyword(&project_id, app_id, query_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn refresh_keywords(
    State(service): State<AppService>,
    Path((project_id, app_id)): Path<(String, i64)>,
    Json(input): Json<RefreshKeyword>,
) -> ApiResult<Json<Value>> {
    let keywords = service
        .refresh_keywords(&project_id, app_id, input.query_id, input.force)
        .await?;
    Ok(Json(json!({"data": keywords})))
}

async fn raw_search(
    State(service): State<AppService>,
    Json(input): Json<SearchRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(serde_json::to_value(
        commands::search::run(service.client(), &input).await?,
    )?))
}

async fn raw_lookup(
    State(service): State<AppService>,
    Json(input): Json<LookupRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(serde_json::to_value(
        commands::lookup::run(service.client(), &input).await?,
    )?))
}

async fn raw_estimates(
    State(service): State<AppService>,
    Json(input): Json<EstimatesRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(serde_json::to_value(
        commands::estimates::run(service.client(), &input).await?,
    )?))
}

async fn raw_popularity(
    State(service): State<AppService>,
    Json(input): Json<PopularityRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(serde_json::to_value(
        commands::popularity::run(service.client(), &input).await?,
    )?))
}

async fn raw_reviews(
    State(service): State<AppService>,
    Json(input): Json<ReviewsRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(serde_json::to_value(
        commands::reviews::run(service.client(), &input).await?,
    )?))
}

async fn raw_chart(
    State(service): State<AppService>,
    Json(input): Json<ChartRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(serde_json::to_value(
        commands::chart::run(service.client(), &input).await?,
    )?))
}

async fn raw_list(Path(resource): Path<String>) -> ApiResult<Json<Value>> {
    let resource = match resource.as_str() {
        "countries" => ListResource::Countries,
        "categories" => ListResource::Categories,
        "chart-types" => ListResource::ChartTypes,
        _ => {
            return Err(ApiError::not_found(format!(
                "unknown list resource '{resource}'"
            )))
        }
    };
    Ok(Json(commands::list::run(resource)?))
}
