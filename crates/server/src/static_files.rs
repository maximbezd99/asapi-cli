use axum::{
    body::Body,
    http::{header, Response, StatusCode, Uri},
    response::IntoResponse,
};
use include_dir::{include_dir, Dir};

static WEB: Dir<'_> = include_dir!("$OUT_DIR/web-dist");

pub async fn serve(uri: Uri) -> impl IntoResponse {
    if uri.path().starts_with("/api/") {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"error":{"status":404,"message":"API endpoint was not found"}}"#,
            ))
            .expect("valid API not found response");
    }
    let requested = uri.path().trim_start_matches('/');
    let requested = if requested.is_empty() {
        "index.html"
    } else {
        requested
    };
    let file = WEB
        .get_file(requested)
        .or_else(|| WEB.get_file("index.html"));
    match file {
        Some(file) => {
            let mime = mime_guess::from_path(file.path()).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(
                    header::CACHE_CONTROL,
                    if requested.starts_with("assets/") {
                        "public, max-age=31536000, immutable"
                    } else {
                        "no-cache"
                    },
                )
                .body(Body::from(file.contents()))
                .expect("valid static response")
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("web application is not bundled"))
            .expect("valid not found response"),
    }
}
