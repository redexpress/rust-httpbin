use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{routing::get, Router};

use crate::error::AppError;
use crate::state::AppState;

pub fn route() -> Router<AppState> {
    Router::new().route("/status/{code}", get(handler))
}

/// `GET /status/:code` — returns the given HTTP status code.
///
/// Valid range: 100–599. Returns 400 for out-of-range values.
#[utoipa::path(get, path = "/status/{code}", responses((status = 200, description = "Return given HTTP status code")))]
pub(crate) async fn handler(
    State(_state): State<AppState>,
    Path(code): Path<u16>,
) -> Result<(StatusCode, &'static str), AppError> {
    if !(100..=599).contains(&code) {
        return Err(AppError::BadRequest(format!(
            "status code {code} is out of range (100–599)"
        )));
    }

    let status = StatusCode::from_u16(code)
        .map_err(|_| AppError::BadRequest(format!("invalid status code: {code}")))?;

    Ok((status, ""))
}
