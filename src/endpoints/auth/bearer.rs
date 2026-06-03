use axum::extract::State;
use axum::http::HeaderMap;
use axum::{routing::get, Router};
use serde::Serialize;

use crate::error::AppError;
use crate::state::AppState;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/bearer", get(handler))
}

#[derive(Serialize)]
struct BearerResponse {
    authenticated: bool,
    token: String,
}

/// `GET /bearer` — validates a Bearer token from the Authorization header.
///
/// Returns 401 if the header is missing or doesn't start with `Bearer `.
#[utoipa::path(get, path = "/bearer", responses((status = 200, description = "Validate Bearer token")))]
pub(crate) async fn handler(
    State(_state): State<AppState>,
    headers: HeaderMap,
) -> Result<axum::response::Response, AppError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization scheme".into()))?;

    Ok(ok_json(&BearerResponse {
        authenticated: true,
        token: token.to_string(),
    }))
}
