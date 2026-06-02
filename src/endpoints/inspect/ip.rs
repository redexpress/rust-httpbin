use axum::extract::State;
use axum::http::HeaderMap;
use axum::{Router, routing::get};
use serde::Serialize;

use crate::state::AppState;
use crate::utils::client_ip::client_ip;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/ip", get(handler))
}

#[derive(Serialize)]
struct IpResponse {
    origin: String,
}

/// `GET /ip` — returns the client's IP address.
async fn handler(
    State(_state): State<AppState>,
    headers: HeaderMap,
) -> axum::response::Response {
    let origin = client_ip(&headers, None);
    ok_json(&IpResponse { origin })
}
