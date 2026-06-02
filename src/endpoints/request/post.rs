use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::{routing::post, Router};
use std::collections::HashMap;

use crate::models::request::RequestInfo;
use crate::state::AppState;
use crate::utils::client_ip::client_ip;
use crate::utils::header_utils::{build_full_url, collect_headers};
use crate::utils::json_utils::{body_as_string, parse_form_body, parse_json_value};
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/post", post(handler))
}

/// `POST /post` — echoes the incoming request including the body.
async fn handler(
    State(state): State<AppState>,
    uri: axum::http::Uri,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> axum::response::Response {
    let info = build_request_info(&uri, &headers, &query, &body);
    let _ = state;
    ok_json(&info)
}

pub(crate) fn build_request_info(
    uri: &axum::http::Uri,
    headers: &HeaderMap,
    query: &HashMap<String, String>,
    body: &Bytes,
) -> RequestInfo {
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let is_json = content_type.contains("application/json");
    let is_form = content_type.contains("application/x-www-form-urlencoded");

    let json = if is_json {
        parse_json_value(body)
    } else {
        None
    };

    let form = if is_form {
        parse_form_body(body)
    } else {
        HashMap::new()
    };

    let data = if is_json || is_form {
        String::new()
    } else {
        body_as_string(body).unwrap_or_default()
    };

    RequestInfo {
        url: build_full_url(headers, uri),
        headers: collect_headers(headers),
        origin: client_ip(headers, None),
        args: query.clone(),
        json,
        data,
        form,
        ..Default::default()
    }
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
    async fn post_parses_json_body() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/post")
            .header("Host", "example.com")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"key":"value"}"#))
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["json"]["key"], "value");
        assert_eq!(json["data"], "");
        assert!(json.get("form").is_some(), "form should always be present");
        assert!(
            json.get("files").is_some(),
            "files should always be present"
        );
    }

    #[tokio::test]
    async fn post_parses_form_body() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/post")
            .header("Host", "example.com")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("foo=bar&baz=qux"))
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["form"]["foo"], "bar");
        assert_eq!(json["form"]["baz"], "qux");
        assert_eq!(json["data"], "");
        assert_eq!(json["json"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn post_raw_text_body() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/post")
            .header("Host", "example.com")
            .header("content-type", "text/plain")
            .body(Body::from("hello world"))
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"], "hello world");
        assert_eq!(json["json"], serde_json::Value::Null);
        assert_eq!(json["form"], serde_json::json!({}));
    }

    #[tokio::test]
    async fn post_always_includes_all_fields() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/post")
            .header("Host", "example.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // All fields must exist even when empty (matches httpbin.org)
        assert!(json.get("url").is_some(), "missing url");
        assert!(json.get("headers").is_some(), "missing headers");
        assert!(json.get("origin").is_some(), "missing origin");
        assert!(json.get("args").is_some(), "missing args");
        assert!(json.get("data").is_some(), "missing data");
        assert!(json.get("json").is_some(), "missing json");
        assert!(json.get("form").is_some(), "missing form");
        assert!(json.get("files").is_some(), "missing files");
    }
}
