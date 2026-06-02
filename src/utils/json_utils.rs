use axum::body::Bytes;
use serde::de::DeserializeOwned;

/// Attempt to deserialize a request body as JSON.
///
/// Returns `None` if the body is empty or not valid JSON.
pub fn parse_json_body<T: DeserializeOwned>(body: &Bytes) -> Option<T> {
    if body.is_empty() {
        return None;
    }
    serde_json::from_slice::<T>(body).ok()
}

/// Attempt to parse a body as a JSON `Value`.
pub fn parse_json_value(body: &Bytes) -> Option<serde_json::Value> {
    parse_json_body::<serde_json::Value>(body)
}

/// Read a body as a UTF-8 string, returning `None` if empty.
pub fn body_as_string(body: &Bytes) -> Option<String> {
    if body.is_empty() {
        return None;
    }
    String::from_utf8(body.to_vec()).ok()
}
