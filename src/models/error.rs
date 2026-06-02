use serde::Serialize;

/// Error response body returned on failure.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
