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

/// Parse an `application/x-www-form-urlencoded` body.
pub fn parse_form_body(body: &Bytes) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if body.is_empty() {
        return map;
    }
    let text = String::from_utf8_lossy(body);
    for pair in text.split('&') {
        if pair.is_empty() {
            continue;
        }
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(
                percent_decode(k).to_string(),
                percent_decode(v).to_string(),
            );
        } else {
            map.insert(percent_decode(pair).to_string(), String::new());
        }
    }
    map
}

fn percent_decode(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains('%') && !s.contains('+') {
        return std::borrow::Cow::Borrowed(s);
    }
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '+' {
            result.push(' ');
        } else if c == '%' {
            let hi = chars.next().and_then(|c| c.to_digit(16));
            let lo = chars.next().and_then(|c| c.to_digit(16));
            if let (Some(h), Some(l)) = (hi, lo) {
                result.push(char::from_u32((h << 4) | l).unwrap_or('\u{FFFD}'));
            }
        } else {
            result.push(c);
        }
    }
    std::borrow::Cow::Owned(result)
}
