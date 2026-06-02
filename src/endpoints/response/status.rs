use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Router, routing::get};

use crate::error::AppError;
use crate::state::AppState;

pub fn route() -> Router<AppState> {
    Router::new().route("/status/{code}", get(handler))
}

/// `GET /status/:code` — returns the given HTTP status code.
///
/// Valid range: 100–599. Returns 400 for out-of-range values.
async fn handler(
    State(_state): State<AppState>,
    Path(code): Path<u16>,
) -> Result<(StatusCode, &'static str), AppError> {
    if !(100..=599).contains(&code) {
        return Err(AppError::BadRequest(format!(
            "status code {} is out of range (100–599)",
            code
        )));
    }

    let status = StatusCode::from_u16(code).map_err(|_| {
        AppError::BadRequest(format!("invalid status code: {}", code))
    })?;

    Ok((status, ""))
}
