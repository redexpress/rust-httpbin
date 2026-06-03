use axum::extract::Query;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::error::AppError;
use crate::state::AppState;

/// Hard cap on the number of response headers a single request may set.
const MAX_HEADERS: usize = 50;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/response-headers", get(handler))
        .route("/response-headers", post(handler))
}

/// Echo of the response headers that were set.
#[derive(Debug, Serialize)]
struct ResponseHeadersEcho {
    #[serde(flatten)]
    headers: BTreeMap<String, String>,
}

/// `GET|POST /response-headers?Key=Value&...`
///
/// Sets each query parameter as a response header and returns the same set
/// as JSON. Header names are validated as HTTP tokens; values are validated
/// as `HeaderValue`. Invalid input → 400.
#[utoipa::path(
    get,
    path = "/response-headers",
    responses((status = 200, description = "Headers echoed back as JSON"))
)]
pub(crate) async fn handler(
    Query(params): Query<BTreeMap<String, String>>,
) -> Result<Response, AppError> {
    if params.len() > MAX_HEADERS {
        return Err(AppError::BadRequest(format!(
            "too many headers: {} (max {})",
            params.len(),
            MAX_HEADERS
        )));
    }

    let mut response_headers = HeaderMap::with_capacity(params.len());
    for (name, value) in &params {
        let header_name = HeaderName::try_from(name.as_str())
            .map_err(|e| AppError::BadRequest(format!("invalid header name '{name}': {e}")))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|e| AppError::BadRequest(format!("invalid header value '{value}': {e}")))?;
        response_headers.insert(header_name, header_value);
    }

    let body = ResponseHeadersEcho {
        headers: params.clone(),
    };
    let mut response = (StatusCode::OK, Json(body)).into_response();
    response.headers_mut().extend(response_headers);
    Ok(response)
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
    async fn sets_and_returns_query_headers() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/response-headers?X-Custom=hello&X-Other=world")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            response.headers().get("X-Custom").unwrap(),
            "hello",
            "X-Custom header missing"
        );
        assert_eq!(
            response.headers().get("X-Other").unwrap(),
            "world",
            "X-Other header missing"
        );

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["X-Custom"], "hello");
        assert_eq!(json["X-Other"], "world");
    }

    #[tokio::test]
    async fn works_with_post() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/response-headers?X-Foo=bar")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("X-Foo").unwrap(), "bar");
    }

    #[tokio::test]
    async fn rejects_invalid_header_name() {
        let app = test_app();
        // Space is not allowed in a header name
        let req = Request::builder()
            .method(Method::GET)
            .uri("/response-headers?Bad%20Name=value")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_invalid_header_value() {
        let app = test_app();
        // Newline in value is invalid for HeaderValue
        let req = Request::builder()
            .method(Method::GET)
            .uri("/response-headers?X-Bad=line1%0Aline2")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn empty_query_returns_empty_json_object() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/response-headers")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json, serde_json::json!({}));
    }
}
