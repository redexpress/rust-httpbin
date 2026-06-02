use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Build a JSON response with a given status code.
pub fn json_response<T: Serialize>(status: StatusCode, body: &T) -> Response {
    (status, Json(body)).into_response()
}

/// Build a 200 OK JSON response.
pub fn ok_json<T: Serialize>(body: &T) -> Response {
    json_response(StatusCode::OK, body)
}

/// Build a 201 Created JSON response.
#[allow(dead_code)]
pub fn created_json<T: Serialize>(body: &T) -> Response {
    json_response(StatusCode::CREATED, body)
}
