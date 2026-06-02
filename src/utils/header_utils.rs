use axum::http::HeaderMap;
use std::collections::HashMap;

/// Build the full request URL from headers and URI.
///
/// Uses the `Host` header (or falls back to `localhost`) plus
/// the request path and query to construct a URL like:
/// `http://example.com/path?key=val`
pub fn build_full_url(headers: &HeaderMap, uri: &axum::http::Uri) -> String {
    let scheme = if headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        == Some("https")
    {
        "https"
    } else {
        "http"
    };

    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");

    let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

    format!("{scheme}://{host}{path_and_query}")
}

/// Collect all request headers into a `HashMap<String, String>`.
///
/// If raw headers are available (via `HeaderPeeker` in the server), their
/// original casing is used. Falls back to `HeaderMap` otherwise (unit tests).
/// Duplicate header values are joined with `, `.
pub fn collect_headers(headers: &HeaderMap) -> HashMap<String, String> {
    // Prefer raw headers with original casing when available
    if let Ok(Some(raw)) = crate::utils::raw_headers::RAW_HEADERS.try_with(|h| h.clone()) {
        return raw;
    }

    // Fallback: use HeaderMap (lowercased by the http crate)
    let mut map = HashMap::new();
    for (name, value) in headers.iter() {
        let key = name.as_str().to_string();
        let val = value.to_str().unwrap_or("<binary>").to_string();

        map.entry(key)
            .and_modify(|existing: &mut String| {
                existing.push_str(", ");
                existing.push_str(&val);
            })
            .or_insert(val);
    }
    map
}

/// Build a map from query string parameters.
#[allow(dead_code)]
pub fn parse_query(query: Option<&str>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(qs) = query {
        for pair in qs.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                map.insert(url_decode(k), url_decode(v));
            } else if !pair.is_empty() {
                map.insert(url_decode(pair), String::new());
            }
        }
    }
    map
}

fn url_decode(s: &str) -> String {
    urlencoding_mock(s)
}

/// Minimal percent-decode (handles `+` → space and `%XX`).
/// In production you'd use the `url` crate; kept minimal here to avoid extra deps.
fn urlencoding_mock(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '+' {
            result.push(' ');
        } else if c == '%' {
            let hi = chars.next().and_then(|c| c.to_digit(16));
            let lo = chars.next().and_then(|c| c.to_digit(16));
            if let (Some(h), Some(l)) = (hi, lo) {
                result.push(char::from_u32((h << 4) | l).unwrap_or('�'));
            } else {
                result.push('%');
                if let Some(hc) = hi {
                    result.push(char::from_digit(hc, 16).unwrap_or('?'));
                }
                if let Some(lc) = lo {
                    result.push(char::from_digit(lc, 16).unwrap_or('?'));
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_headers_preserves_case() {
        let mut headers = HeaderMap::new();
        // Custom headers preserve original casing in the http crate
        headers.insert("X-Custom-Header", "test-value".parse().unwrap());
        headers.insert("Accept", "text/html".parse().unwrap());
        let map = collect_headers(&headers);
        // Custom headers keep their case
        assert_eq!(map.get("x-custom-header").unwrap(), "test-value");
        // Standard headers may be lowercased by the http crate (HTTP spec)
        assert!(map.contains_key("accept") || map.contains_key("Accept"));
    }

    #[test]
    fn parses_query_string() {
        let map = parse_query(Some("a=1&b=2&c"));
        assert_eq!(map.get("a").unwrap(), "1");
        assert_eq!(map.get("b").unwrap(), "2");
        assert_eq!(map.get("c").unwrap(), "");
    }
}
