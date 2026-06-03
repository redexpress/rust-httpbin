use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::{routing::get, Router};
use std::collections::HashMap;

use crate::models::response::GetResponse;
use crate::state::AppState;
use crate::utils::client_ip::client_ip;
use crate::utils::header_utils::{build_full_url, collect_headers};
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/get", get(handler))
}

/// `GET /get` — echoes the incoming request.
#[utoipa::path(get, path = "/get", responses((status = 200, description = "Echo the GET request")))]
pub(crate) async fn handler(
    State(_state): State<AppState>,
    uri: axum::http::Uri,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> axum::response::Response {
    ok_json(&GetResponse {
        url: build_full_url(&headers, &uri),
        headers: collect_headers(&headers),
        origin: client_ip(&headers, None),
        args: query,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    fn test_app() -> axum::Router {
        crate::app::build_app(AppState::new())
    }

    #[tokio::test]
    async fn get_returns_200() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/get")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn get_has_only_four_fields() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/get?foo=bar")
            .header("Host", "example.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Exactly 4 fields — matches httpbin.org/get
        assert_eq!(json.as_object().unwrap().len(), 4);
        assert!(json.get("args").is_some(), "missing args");
        assert!(json.get("headers").is_some(), "missing headers");
        assert!(json.get("origin").is_some(), "missing origin");
        assert!(json.get("url").is_some(), "missing url");
    }

    #[tokio::test]
    async fn get_url_is_full_url() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/get?a=1")
            .header("Host", "example.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["url"], "http://example.com/get?a=1");
    }

    #[tokio::test]
    async fn get_echos_query_params() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/get?x=1&y=hello")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["args"]["x"], "1");
        assert_eq!(json["args"]["y"], "hello");
    }

    #[tokio::test]
    async fn get_echos_headers() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/get")
            .header("X-Custom", "test-value")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["headers"]["x-custom"], "test-value");
    }
}
