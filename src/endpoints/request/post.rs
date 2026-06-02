use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::{Router, routing::post};
use std::collections::HashMap;

use crate::models::request::RequestInfo;
use crate::state::AppState;
use crate::utils::client_ip::client_ip;
use crate::utils::header_utils::collect_headers;
use crate::utils::json_utils::{body_as_string, parse_json_value};
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/post", post(handler))
}

/// `POST /post` — echoes the incoming request including the body.
async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> axum::response::Response {
    let info = build_request_info("POST", "/post", &headers, &query, &body);
    let _ = state;
    ok_json(&info)
}

pub(crate) fn build_request_info(
    method: &str,
    url: &str,
    headers: &HeaderMap,
    query: &HashMap<String, String>,
    body: &Bytes,
) -> RequestInfo {
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let json = if content_type.contains("application/json") {
        parse_json_value(body)
    } else {
        None
    };

    let data = if json.is_none() {
        body_as_string(body)
    } else {
        None
    };

    RequestInfo {
        method: method.to_string(),
        url: url.to_string(),
        headers: collect_headers(headers),
        origin: client_ip(headers, None),
        args: query.clone(),
        json,
        data,
        form: HashMap::new(),
        files: HashMap::new(),
    }
}
