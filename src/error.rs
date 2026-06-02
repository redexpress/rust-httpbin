use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::models::error::ErrorResponse;

/// Application-wide error type.
///
/// Every endpoint that can fail should return `Result<T, AppError>`.
/// Axum will convert this into an HTTP response automatically via `IntoResponse`.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    /// 400 — bad input (missing/invalid parameter)
    BadRequest(String),

    /// 401 — missing or invalid credentials
    Unauthorized(String),

    /// 404 — resource not found
    NotFound(String),

    /// 500 — unexpected internal error
    Internal(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::BadRequest(m)
            | Self::Unauthorized(m)
            | Self::NotFound(m)
            | Self::Internal(m) => m,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(ErrorResponse {
            error: status.to_string(),
            message: self.message().to_string(),
        });
        (status, body).into_response()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status_code(), self.message())
    }
}

impl std::error::Error for AppError {}
