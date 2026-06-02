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
        .header("Host", "example.com")
        .header("x-custom", "test-value")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["url"], "http://example.com/get?foo=bar");
    assert_eq!(json["args"]["foo"], "bar");
    assert_eq!(json["headers"]["x-custom"], "test-value");

    // GET has only 4 fields — matches httpbin.org
    assert_eq!(json.as_object().unwrap().len(), 4);
}

#[tokio::test]
async fn post_echoes_body() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::POST)
        .uri("/post")
        .header("Host", "example.com")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"key":"value"}"#))
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["url"], "http://example.com/post");
    assert_eq!(json["json"]["key"], "value");
    assert!(json.get("method").is_none(), "method should not be present");
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
