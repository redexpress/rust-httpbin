use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;

use axum_httpbin::app;
use axum_httpbin::state::AppState;

#[tokio::test]
async fn delay_returns_after_wait() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/delay/0.1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["delay_secs"], 0.1);
}

#[tokio::test]
async fn delay_rejects_negative() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/delay/-1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn redirect_requires_url() {
    let app = app::build_app(AppState::new());
    let req = Request::builder()
        .method(Method::GET)
        .uri("/redirect-to")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
