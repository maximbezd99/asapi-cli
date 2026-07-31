use serde_json::{json, Map, Value};

pub fn document() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "asapi local application API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Manage local App Store research projects and call the public App Store query operations."
        },
        "servers": [{"url": "/api/v1"}],
        "paths": {
            "/projects": {
                "get": operation("List projects", None, "200"),
                "post": operation("Create a project", Some("CreateProject"), "201")
            },
            "/projects/{project_id}": {
                "parameters": [path_parameter("project_id", "Project UUID")],
                "get": operation("Get a project", None, "200"),
                "patch": operation("Rename a project", Some("RenameProject"), "200"),
                "delete": operation("Delete a project", None, "200")
            },
            "/projects/{project_id}/apps": {
                "parameters": [path_parameter("project_id", "Project UUID")],
                "get": operation("List tracked apps", None, "200"),
                "post": operation("Track an app and fetch its main storefront", Some("AddApp"), "201")
            },
            "/projects/{project_id}/apps/{app_id}": {
                "parameters": [
                    path_parameter("project_id", "Project UUID"),
                    path_parameter("app_id", "Apple App Store ID"),
                    query_parameter("country", "Configured storefront country code", false)
                ],
                "get": operation("Get the stored app view for a storefront", None, "200"),
                "delete": operation("Stop tracking an app", None, "204")
            },
            "/projects/{project_id}/apps/{app_id}/refresh": {
                "parameters": [
                    path_parameter("project_id", "Project UUID"),
                    path_parameter("app_id", "Apple App Store ID")
                ],
                "post": operation("Refresh one storefront or all automatic storefronts", Some("RefreshApp"), "200")
            },
            "/projects/{project_id}/apps/{app_id}/history": {
                "parameters": [
                    path_parameter("project_id", "Project UUID"),
                    path_parameter("app_id", "Apple App Store ID"),
                    query_parameter("country", "Configured storefront country code", false),
                    {
                        "name": "resource",
                        "in": "query",
                        "required": false,
                        "schema": {
                            "type": "string",
                            "enum": ["details", "estimates", "popularity"],
                            "default": "details"
                        }
                    }
                ],
                "get": operation("Read up to 30 days of stored app observations", None, "200")
            },
            "/projects/{project_id}/apps/{app_id}/storefronts": {
                "parameters": [
                    path_parameter("project_id", "Project UUID"),
                    path_parameter("app_id", "Apple App Store ID")
                ],
                "get": operation("List configured storefronts", None, "200"),
                "post": operation("Add a storefront", Some("AddStorefront"), "201")
            },
            "/projects/{project_id}/apps/{app_id}/storefronts/{country}": {
                "parameters": [
                    path_parameter("project_id", "Project UUID"),
                    path_parameter("app_id", "Apple App Store ID"),
                    path_parameter("country", "Configured storefront country code")
                ],
                "patch": operation("Change main or automatic storefront settings", Some("UpdateStorefront"), "200"),
                "delete": operation("Remove a non-main storefront", None, "204")
            },
            "/projects/{project_id}/apps/{app_id}/reviews": {
                "parameters": [
                    path_parameter("project_id", "Project UUID"),
                    path_parameter("app_id", "Apple App Store ID"),
                    query_parameter("country", "Configured storefront country code", true),
                    integer_query_parameter("page", "Review result page, from 1 to 10", false),
                    integer_query_parameter("rating", "Optional star rating, from 1 to 5", false)
                ],
                "get": operation("Read and lazily refresh one review page", None, "200")
            },
            "/projects/{project_id}/apps/{app_id}/keywords": {
                "parameters": [
                    path_parameter("project_id", "Project UUID"),
                    path_parameter("app_id", "Apple App Store ID")
                ],
                "get": operation("List tracked keywords with ranking history", None, "200"),
                "post": operation("Track a storefront-specific keyword", Some("AddKeyword"), "201")
            },
            "/projects/{project_id}/apps/{app_id}/keywords/{query_id}": {
                "parameters": [
                    path_parameter("project_id", "Project UUID"),
                    path_parameter("app_id", "Apple App Store ID"),
                    path_parameter("query_id", "Shared keyword query ID")
                ],
                "patch": operation("Update keyword notes", Some("UpdateKeyword"), "200"),
                "delete": operation("Stop tracking a keyword", None, "204")
            },
            "/projects/{project_id}/apps/{app_id}/keywords/refresh": {
                "parameters": [
                    path_parameter("project_id", "Project UUID"),
                    path_parameter("app_id", "Apple App Store ID")
                ],
                "post": operation("Refresh shared keyword query caches", Some("RefreshKeyword"), "200")
            },
            "/query/search": {
                "post": operation("Equivalent to asapi search", Some("SearchRequest"), "200")
            },
            "/query/lookup": {
                "post": operation("Equivalent to asapi lookup", Some("LookupRequest"), "200")
            },
            "/query/popularity": {
                "post": operation("Equivalent to asapi popularity", Some("PopularityRequest"), "200")
            },
            "/query/reviews": {
                "post": operation("Equivalent to asapi reviews", Some("ReviewsRequest"), "200")
            },
            "/query/chart": {
                "post": operation("Equivalent to asapi chart", Some("ChartRequest"), "200")
            },
            "/query/list/{resource}": {
                "parameters": [{
                    "name": "resource",
                    "in": "path",
                    "required": true,
                    "schema": {
                        "type": "string",
                        "enum": ["countries", "categories", "chart-types"]
                    }
                }],
                "get": operation("Equivalent to asapi list", None, "200")
            }
        },
        "components": {
            "schemas": schemas()
        }
    })
}

fn operation(summary: &str, request_schema: Option<&str>, success: &str) -> Value {
    let mut operation = Map::from_iter([
        ("summary".to_string(), json!(summary)),
        (
            "responses".to_string(),
            json!({
                success: {
                    "description": if success == "204" { "No content" } else { "Success" },
                    "content": if success == "204" {
                        Value::Null
                    } else {
                        json!({"application/json": {"schema": {}}})
                    }
                },
                "400": {
                    "description": "Invalid request",
                    "content": {
                        "application/json": {
                            "schema": {"$ref": "#/components/schemas/Error"}
                        }
                    }
                }
            }),
        ),
    ]);
    if let Some(schema) = request_schema {
        operation.insert(
            "requestBody".to_string(),
            json!({
                "required": true,
                "content": {
                    "application/json": {
                        "schema": {"$ref": format!("#/components/schemas/{schema}")}
                    }
                }
            }),
        );
    }
    Value::Object(operation)
}

fn path_parameter(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "in": "path",
        "required": true,
        "description": description,
        "schema": if name.ends_with("_id") && name != "project_id" {
            json!({"type": "integer"})
        } else {
            json!({"type": "string"})
        }
    })
}

fn query_parameter(name: &str, description: &str, required: bool) -> Value {
    json!({
        "name": name,
        "in": "query",
        "required": required,
        "description": description,
        "schema": {"type": "string"}
    })
}

fn integer_query_parameter(name: &str, description: &str, required: bool) -> Value {
    json!({
        "name": name,
        "in": "query",
        "required": required,
        "description": description,
        "schema": {"type": "integer"}
    })
}

fn schemas() -> Value {
    json!({
        "Error": {
            "type": "object",
            "properties": {
                "error": {
                    "type": "object",
                    "properties": {
                        "status": {"type": "integer"},
                        "message": {"type": "string"}
                    },
                    "required": ["status", "message"]
                }
            },
            "required": ["error"]
        },
        "AppSpecifier": {
            "type": "object",
            "properties": {
                "id": {"type": "integer", "minimum": 1},
                "country": {"type": ["string", "null"]}
            },
            "required": ["id"]
        },
        "CreateProject": object(json!({"name": {"type": "string"}}), &["name"]),
        "RenameProject": object(json!({"name": {"type": "string"}}), &["name"]),
        "AddApp": object(
            json!({
                "app_id": {"type": "integer", "minimum": 1},
                "country": {"type": "string", "default": "us"}
            }),
            &["app_id"]
        ),
        "RefreshApp": object(
            json!({
                "country": {"type": ["string", "null"]},
                "all": {
                    "type": "boolean",
                    "default": false,
                    "description": "Refresh every configured storefront instead of the main and automatic storefronts."
                }
            }),
            &[]
        ),
        "AddStorefront": object(
            json!({
                "country": {"type": "string"},
                "auto_refresh": {"type": "boolean", "default": false}
            }),
            &["country"]
        ),
        "UpdateStorefront": object(
            json!({
                "is_main": {"type": ["boolean", "null"]},
                "auto_refresh": {"type": ["boolean", "null"]}
            }),
            &[]
        ),
        "AddKeyword": object(
            json!({
                "keyword": {"type": "string"},
                "country": {"type": "string"},
                "notes": {"type": "string", "default": ""}
            }),
            &["keyword", "country"]
        ),
        "UpdateKeyword": object(json!({"notes": {"type": "string"}}), &["notes"]),
        "RefreshKeyword": object(
            json!({
                "query_id": {"type": ["integer", "null"]},
                "force": {"type": "boolean", "default": false}
            }),
            &[]
        ),
        "SearchRequest": object(
            json!({
                "term": {"type": "string"},
                "country": {"type": ["string", "null"]},
                "limit": {"type": "integer", "default": 10, "maximum": 200},
                "local_limit": {"type": ["integer", "null"], "maximum": 200}
            }),
            &["term"]
        ),
        "LookupRequest": object(
            json!({
                "apps": {
                    "type": "array",
                    "items": {"$ref": "#/components/schemas/AppSpecifier"},
                    "maxItems": 10
                },
                "country": {"type": ["string", "null"]},
                "full": {
                    "type": "boolean",
                    "default": false,
                    "description": "Fetch product-page data plus worldwide last-month download and revenue estimates."
                }
            }),
            &["apps"]
        ),
        "PopularityRequest": object(
            json!({
                "app": {"$ref": "#/components/schemas/AppSpecifier"},
                "group": {"type": "string", "enum": ["tier1", "tier2"], "default": "tier1"},
                "countries": {
                    "type": ["array", "null"],
                    "items": {"type": "string"}
                }
            }),
            &["app"]
        ),
        "ReviewsRequest": object(
            json!({
                "app": {"$ref": "#/components/schemas/AppSpecifier"},
                "country": {"type": ["string", "null"]},
                "page": {"type": ["integer", "null"], "minimum": 1, "maximum": 10},
                "pages": {"type": ["integer", "null"], "minimum": 1, "maximum": 10},
                "all": {"type": "boolean", "default": false}
            }),
            &["app"]
        ),
        "ChartRequest": object(
            json!({
                "chart": {"type": "string", "enum": ["top", "free", "paid", "grossing"]},
                "country": {"type": ["string", "null"]},
                "limit": {"type": "integer", "default": 10, "maximum": 200},
                "category": {"type": ["integer", "null"]}
            }),
            &["chart"]
        )
    })
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}
