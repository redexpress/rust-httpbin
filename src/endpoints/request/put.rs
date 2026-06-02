use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::{routing::put, Router};
use std::collections::HashMap;

use crate::endpoints::request::post::build_request_info;
use crate::state::AppState;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/put", put(handler))
}

/// `PUT /put` — echoes the incoming PUT request including the body.
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
