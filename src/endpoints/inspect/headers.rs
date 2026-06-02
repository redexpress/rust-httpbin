use axum::extract::State;
use axum::http::HeaderMap;
use axum::{Router, routing::get};
use crate::state::AppState;
use crate::utils::header_utils::collect_headers;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/headers", get(handler))
}

/// `GET /headers` — returns the request headers as JSON.
async fn handler(
    State(_state): State<AppState>,
    headers: HeaderMap,
) -> axum::response::Response {
    let map = collect_headers(&headers);
    ok_json(&map)
}
