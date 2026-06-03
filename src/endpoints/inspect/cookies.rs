use axum::extract::{Path, Query};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use cookie::Cookie;
use serde::Serialize;
use std::collections::HashMap;

use crate::error::AppError;
use crate::state::AppState;

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/cookies", get(get_cookies))
        .route("/cookies/set", get(set_cookies))
        .route("/cookies/set/{name}/{value}", get(set_cookie_one))
        .route("/cookies/delete", get(delete_cookies))
}

/// Cookie jar returned to the client.
#[derive(Debug, Serialize)]
struct CookiesResponse {
    #[serde(flatten)]
    cookies: HashMap<String, String>,
}

/// `GET /cookies` — return cookies sent by the client as a JSON object.
#[utoipa::path(get, path = "/cookies", responses((status = 200, description = "Return cookies as JSON")))]
pub(crate) async fn get_cookies(headers: HeaderMap) -> Response {
    let jar = parse_cookie_header(&headers);
    (StatusCode::OK, Json(CookiesResponse { cookies: jar })).into_response()
}

/// `GET /cookies/set?name=value&name2=value2` — set one or more cookies,
/// then 302 to `/cookies`.
#[utoipa::path(get, path = "/cookies/set", responses((status = 302, description = "Set cookies and redirect")))]
pub(crate) async fn set_cookies(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    if params.is_empty() {
        return Err(AppError::BadRequest(
            "expected at least one name=value query parameter".into(),
        ));
    }
    let cookies = build_set_cookies(&params)?;
    Ok(redirect_with_cookies("/cookies", cookies))
}

/// `GET /cookies/set/{name}/{value}` — set a single cookie, 302 to `/cookies`.
#[utoipa::path(get, path = "/cookies/set/{name}/{value}", responses((status = 302, description = "Set cookie and redirect")))]
pub(crate) async fn set_cookie_one(
    Path((name, value)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let cookies = build_set_cookies(&HashMap::from([(name, value)]))?;
    Ok(redirect_with_cookies("/cookies", cookies))
}

/// `GET /cookies/delete?name1&name2` — clear cookies with `Set-Cookie: name=; Max-Age=0`,
/// then 302 to `/cookies`.
#[utoipa::path(get, path = "/cookies/delete", responses((status = 302, description = "Delete cookies and redirect")))]
pub(crate) async fn delete_cookies(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    if params.is_empty() {
        return Err(AppError::BadRequest(
            "expected at least one name query parameter".into(),
        ));
    }
    let mut cookies = Vec::with_capacity(params.len());
    for name in params.keys() {
        if name.is_empty() {
            return Err(AppError::BadRequest("cookie name cannot be empty".into()));
        }
        let mut c = Cookie::build((name.clone(), String::new()))
            .path("/")
            .build();
        c.make_removal();
        cookies.push(c);
    }
    Ok(redirect_with_cookies("/cookies", cookies))
}

// ---------- helpers ----------

fn parse_cookie_header(headers: &HeaderMap) -> HashMap<String, String> {
    let mut jar = HashMap::new();
    let Some(value) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) else {
        return jar;
    };
    for pair in value.split(';') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        if let Some((k, v)) = pair.split_once('=') {
            jar.insert(k.trim().to_string(), v.trim().to_string());
        } else {
            jar.insert(pair.to_string(), String::new());
        }
    }
    jar
}

fn build_set_cookies(pairs: &HashMap<String, String>) -> Result<Vec<Cookie<'static>>, AppError> {
    let mut cookies = Vec::with_capacity(pairs.len());
    for (name, value) in pairs {
        if name.is_empty() {
            return Err(AppError::BadRequest("cookie name cannot be empty".into()));
        }
        let c = Cookie::build((name.clone(), value.clone()))
            .path("/")
            .http_only(false)
            .build();
        cookies.push(c);
    }
    Ok(cookies)
}

fn redirect_with_cookies(location: &str, cookies: Vec<Cookie<'static>>) -> Response {
    let mut response = (StatusCode::FOUND, "").into_response();
    if let Ok(value) = location.parse() {
        response.headers_mut().insert(header::LOCATION, value);
    }
    for cookie in cookies {
        if let Ok(value) = cookie.to_string().parse() {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    fn test_app() -> axum::Router {
        crate::app::build_app(AppState::new())
    }

    #[tokio::test]
    async fn get_cookies_parses_request_cookie_header() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/cookies")
            .header("Cookie", "a=1; b=2")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["a"], "1");
        assert_eq!(json["b"], "2");
    }

    #[tokio::test]
    async fn get_cookies_returns_empty_when_no_cookie_header() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/cookies")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json, serde_json::json!({}));
    }

    #[tokio::test]
    async fn set_cookie_one_sets_and_redirects() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/cookies/set/session/abc123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/cookies"
        );
        let set_cookie = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .next()
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            set_cookie.starts_with("session=abc123"),
            "got: {set_cookie}"
        );
    }

    #[tokio::test]
    async fn set_cookies_handles_multiple_query_params() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/cookies/set?x=1&y=2")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::FOUND);
        let set_cookies: Vec<&str> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect();
        assert_eq!(set_cookies.len(), 2);
        let joined = set_cookies.join("|");
        assert!(joined.contains("x=1"), "missing x=1: {joined}");
        assert!(joined.contains("y=2"), "missing y=2: {joined}");
    }

    #[tokio::test]
    async fn set_cookies_rejects_empty_query() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/cookies/set")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_cookies_clears_and_redirects() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/cookies/delete?x&y")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::FOUND);
        let set_cookies: Vec<&str> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect();
        assert_eq!(set_cookies.len(), 2);
        let joined = set_cookies.join("|");
        assert!(joined.contains("x="), "missing x= : {joined}");
        assert!(joined.contains("y="), "missing y= : {joined}");
        assert!(
            joined.to_lowercase().contains("max-age=0"),
            "missing Max-Age=0: {joined}"
        );
    }

    #[tokio::test]
    async fn delete_cookies_rejects_empty_query() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/cookies/delete")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
