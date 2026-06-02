use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Router, routing::get};
use serde::Deserialize;

use crate::error::AppError;
use crate::state::AppState;

pub fn route() -> Router<AppState> {
    Router::new().route("/redirect-to", get(handler))
}

#[derive(Deserialize)]
struct RedirectParams {
    url: Option<String>,
    status_code: Option<u16>,
}

/// `GET /redirect-to?url=...&status_code=...` — redirects to the given URL.
///
/// Default status code: 302. Supports 301, 302, 303, 307, 308.
async fn handler(
    State(_state): State<AppState>,
    Query(params): Query<RedirectParams>,
) -> Result<(StatusCode, HeaderMap, ()), AppError> {
    let url = params.url.ok_or_else(|| {
        AppError::BadRequest("query parameter 'url' is required".into())
    })?;

    let status_code = params.status_code.unwrap_or(302);
    let status = match status_code {
        301 => StatusCode::MOVED_PERMANENTLY,
        302 => StatusCode::FOUND,
        303 => StatusCode::SEE_OTHER,
        307 => StatusCode::TEMPORARY_REDIRECT,
        308 => StatusCode::PERMANENT_REDIRECT,
        _ => {
            return Err(AppError::BadRequest(format!(
                "unsupported redirect status code: {}. Supported: 301, 302, 303, 307, 308",
                status_code
            )))
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert("location", url.parse().unwrap());

    Ok((status, headers, ()))
}
