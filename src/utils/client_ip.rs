use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use std::net::SocketAddr;

/// Resolve the best-guess client IP address.
///
/// Priority:
/// 1. `X-Forwarded-For` header (leftmost entry)
/// 2. `X-Real-Ip` header
/// 3. `ConnectInfo` (direct peer address)
pub fn client_ip(headers: &HeaderMap, connect_info: Option<&ConnectInfo<SocketAddr>>) -> String {
    // Try X-Forwarded-For first
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(val) = xff.to_str() {
            // Take the leftmost (original client) IP
            if let Some(ip) = val.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // Try X-Real-Ip
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(val) = xri.to_str() {
            return val.trim().to_string();
        }
    }

    // Fall back to direct connection
    if let Some(ci) = connect_info {
        return ci.ip().to_string();
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "10.0.0.1, 10.0.0.2".parse().unwrap());
        assert_eq!(client_ip(&headers, None), "10.0.0.1");
    }

    #[test]
    fn returns_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "10.0.0.3".parse().unwrap());
        assert_eq!(client_ip(&headers, None), "10.0.0.3");
    }

    #[test]
    fn returns_unknown_without_info() {
        let headers = HeaderMap::new();
        assert_eq!(client_ip(&headers, None), "unknown");
    }
}
