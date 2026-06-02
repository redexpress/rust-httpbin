use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::{Router, routing::any};
use std::collections::HashMap;

use crate::models::request::RequestInfo;
use crate::state::AppState;
use crate::utils::client_ip::client_ip;
use crate::utils::header_utils::{build_full_url, collect_headers};
use crate::utils::json_utils::{body_as_string, parse_json_value};
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/anything", any(handler))
        .route("/anything/{*path}", any(handler))
}

/// `ANY /anything[/...]` — catch-all that echoes the entire request.
async fn handler(
    State(_state): State<AppState>,
    uri: axum::http::Uri,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> axum::response::Response {
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let json = if content_type.contains("application/json") {
        parse_json_value(&body)
    } else {
        None
    };

    let data = if json.is_none() {
        body_as_string(&body)
    } else {
        None
    };

    ok_json(&RequestInfo {
        url: build_full_url(&headers, &uri),
        headers: collect_headers(&headers),
        origin: client_ip(&headers, None),
        args: query,
        json,
        data,
        ..Default::default()
    })
}
