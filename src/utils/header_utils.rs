use axum::http::HeaderMap;
use std::collections::HashMap;

/// Collect all request headers into a `HashMap<String, String>`.
///
/// Header names are lowercased for consistency.
/// If a header appears multiple times, values are joined with `, `.
pub fn collect_headers(headers: &HeaderMap) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for (name, value) in headers.iter() {
        let key = name.as_str().to_lowercase();
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
                map.insert(
                    url_decode(k),
                    url_decode(v),
                );
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
    fn collects_headers_lowercase() {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let map = collect_headers(&headers);
        assert_eq!(map.get("content-type").unwrap(), "application/json");
    }

    #[test]
    fn parses_query_string() {
        let map = parse_query(Some("a=1&b=2&c"));
        assert_eq!(map.get("a").unwrap(), "1");
        assert_eq!(map.get("b").unwrap(), "2");
        assert_eq!(map.get("c").unwrap(), "");
    }
}
