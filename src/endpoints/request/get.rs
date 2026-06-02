use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::{Router, routing::get};
use std::collections::HashMap;

use crate::models::request::RequestInfo;
use crate::state::AppState;
use crate::utils::client_ip::client_ip;
use crate::utils::header_utils::collect_headers;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/get", get(handler))
}

/// `GET /get` — echoes the incoming request.
async fn handler(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> axum::response::Response {
    ok_json(&RequestInfo {
        method: "GET".to_string(),
        url: "/get".to_string(),
        headers: collect_headers(&headers),
        origin: client_ip(&headers, None),
        args: query,
        json: None,
        data: None,
        form: HashMap::new(),
        files: HashMap::new(),
    })
}
