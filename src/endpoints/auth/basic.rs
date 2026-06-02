use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::{routing::get, Router};
use serde::Serialize;

use crate::error::AppError;
use crate::state::AppState;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/basic-auth/{user}/{passwd}", get(handler))
}

#[derive(Serialize)]
struct AuthResponse {
    authenticated: bool,
    user: String,
}

/// `GET /basic-auth/:user/:passwd` — validates HTTP Basic auth credentials.
///
/// Returns 401 if the `Authorization` header is missing or doesn't match.
async fn handler(
    State(_state): State<AppState>,
    Path((user, passwd)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<axum::response::Response, AppError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    let credentials = auth_header
        .strip_prefix("Basic ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization scheme".into()))?;

    let decoded = base64_decode(credentials)
        .map_err(|_| AppError::Unauthorized("Invalid Base64 encoding".into()))?;

    let (provided_user, provided_pass) = decoded
        .split_once(':')
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials format".into()))?;

    if provided_user != user || provided_pass != passwd {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    Ok(ok_json(&AuthResponse {
        authenticated: true,
        user: provided_user.to_string(),
    }))
}

fn base64_decode(input: &str) -> Result<String, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .map_err(|e| e.to_string())
}
