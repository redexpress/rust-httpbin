use axum::extract::{Path, State};
use axum::{routing::get, Router};
use serde::Serialize;
use std::time::Duration;

use crate::error::AppError;
use crate::state::AppState;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/delay/{secs}", get(handler))
}

#[derive(Serialize)]
struct DelayResponse {
    delay_secs: f64,
    message: String,
}

/// `GET /delay/:secs` — waits N seconds then responds.
///
/// Max delay: 60 seconds. Returns 400 for negative or excessive values.
async fn handler(
    State(_state): State<AppState>,
    Path(secs): Path<f64>,
) -> Result<axum::response::Response, AppError> {
    if secs < 0.0 {
        return Err(AppError::BadRequest("delay must be non-negative".into()));
    }
    if secs > 60.0 {
        return Err(AppError::BadRequest(
            "delay must not exceed 60 seconds".into(),
        ));
    }

    tokio::time::sleep(Duration::from_secs_f64(secs)).await;

    Ok(ok_json(&DelayResponse {
        delay_secs: secs,
        message: format!("waited {secs:.3} seconds"),
    }))
}
