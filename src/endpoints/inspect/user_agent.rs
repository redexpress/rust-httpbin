use axum::extract::State;
use axum::http::HeaderMap;
use axum::{routing::get, Router};
use serde::Serialize;

use crate::state::AppState;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/user-agent", get(handler))
}

#[derive(Serialize)]
struct UserAgentResponse {
    #[serde(rename = "user-agent")]
    user_agent: String,
}

/// `GET /user-agent` — returns the client's User-Agent header.
async fn handler(State(_state): State<AppState>, headers: HeaderMap) -> axum::response::Response {
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    ok_json(&UserAgentResponse { user_agent: ua })
}
