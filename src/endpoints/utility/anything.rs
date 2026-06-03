use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::{routing::any, Router};
use std::collections::HashMap;

use crate::endpoints::request::post::build_request_info;
use crate::state::AppState;
use crate::utils::request_body::RequestBody;
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
    body: RequestBody,
) -> axum::response::Response {
    let info = build_request_info(&uri, &headers, &query, &body);
    ok_json(&info)
}
