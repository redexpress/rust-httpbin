use axum::extract::State;
use axum::{routing::get, Router};

use crate::models::response::UuidResponse;
use crate::state::AppState;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/uuid", get(handler))
}

/// `GET /uuid` — returns a randomly-generated v4 UUID.
async fn handler(State(_state): State<AppState>) -> axum::response::Response {
    let id = uuid::Uuid::new_v4().to_string();
    ok_json(&UuidResponse { uuid: id })
}
