use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;

use axum_httpbin::app;
use axum_httpbin::state::AppState;

#[tokio::test]
async fn headers_returns_all_headers() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/headers")
        .header("x-foo", "bar")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["x-foo"], "bar");
}

#[tokio::test]
async fn ip_returns_origin() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/ip")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["origin"].is_string());
}

#[tokio::test]
async fn uuid_returns_valid_uuid() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/uuid")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let uuid_str = json["uuid"].as_str().unwrap();
    assert_eq!(uuid_str.len(), 36); // standard UUIDv4 length
    assert_eq!(uuid_str.chars().filter(|&c| c == '-').count(), 4);
}

#[tokio::test]
async fn anything_echoes_request() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::POST)
        .uri("/anything/some/path?q=1")
        .header("content-type", "text/plain")
        .body(Body::from("hello world"))
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["method"], "POST");
    assert_eq!(json["data"], "hello world");
}
