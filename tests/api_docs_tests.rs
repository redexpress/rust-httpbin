use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;

use axum_httpbin::app;
use axum_httpbin::state::AppState;

#[tokio::test]
async fn root_serves_swagger_ui_html() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let html = std::str::from_utf8(&body).expect("response should be utf-8");
    assert!(
        html.contains("swagger-ui"),
        "expected swagger-ui in / response, got: {html}"
    );
}

#[tokio::test]
async fn openapi_json_returns_valid_spec() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/openapi.json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let spec: serde_json::Value =
        serde_json::from_slice(&body).expect("/openapi.json must be valid JSON");

    assert!(spec["openapi"].is_string(), "missing openapi version field");
    assert_eq!(spec["info"]["title"], "Axum Httpbin");
    assert!(spec["paths"]["/get"].is_object(), "missing /get path");
}

#[tokio::test]
async fn openapi_lists_all_routes() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/openapi.json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let spec: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let paths = spec["paths"]
        .as_object()
        .expect("paths should be an object");

    for path in [
        "/get",
        "/post",
        "/put",
        "/patch",
        "/delete",
        "/headers",
        "/ip",
        "/user-agent",
        "/status/{code}",
        "/delay/{secs}",
        "/redirect-to",
        "/stream/{n}",
        "/basic-auth/{user}/{passwd}",
        "/bearer",
        "/uuid",
        "/image/png",
        "/image/jpeg",
        "/image/webp",
    ] {
        assert!(
            paths.contains_key(path),
            "missing path {path} in OpenAPI spec; got paths: {:?}",
            paths.keys().collect::<Vec<_>>()
        );
    }
}
