use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;

use axum_httpbin::app;
use axum_httpbin::state::AppState;

#[tokio::test]
async fn get_echoes_request() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/get?foo=bar")
        .header("x-custom", "test-value")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["method"], "GET");
    assert_eq!(json["args"]["foo"], "bar");
    assert_eq!(json["headers"]["x-custom"], "test-value");
}

#[tokio::test]
async fn post_echoes_body() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::POST)
        .uri("/post")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"key":"value"}"#))
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["method"], "POST");
    assert_eq!(json["json"]["key"], "value");
}

#[tokio::test]
async fn status_code_echoes() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/status/418")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::IM_A_TEAPOT);
}
