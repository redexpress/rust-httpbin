use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;

use crate::state::AppState;
use crate::utils::image_bytes::{JPEG_BYTES, PNG_BYTES, SVG_BYTES, WEBP_BYTES};

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/image", get(serve_png))
        .route("/image/png", get(serve_png))
        .route("/image/jpeg", get(serve_jpeg))
        .route("/image/webp", get(serve_webp))
        .route("/image/svg", get(serve_svg))
}

/// `GET /image[/png]` — returns the embedded PNG with `image/png`.
#[utoipa::path(get, path = "/image/png", responses((status = 200, description = "Return a PNG image")))]
pub(crate) async fn serve_png(State(_state): State<AppState>) -> Response {
    build_image_response("image/png", PNG_BYTES)
}

/// `GET /image/jpeg` — returns the embedded JPEG with `image/jpeg`.
#[utoipa::path(get, path = "/image/jpeg", responses((status = 200, description = "Return a JPEG image")))]
pub(crate) async fn serve_jpeg(State(_state): State<AppState>) -> Response {
    build_image_response("image/jpeg", JPEG_BYTES)
}

/// `GET /image/webp` — returns the embedded WebP with `image/webp`.
#[utoipa::path(get, path = "/image/webp", responses((status = 200, description = "Return a WebP image")))]
pub(crate) async fn serve_webp(State(_state): State<AppState>) -> Response {
    build_image_response("image/webp", WEBP_BYTES)
}

/// `GET /image/svg` — returns the embedded SVG with `image/svg+xml`.
#[utoipa::path(get, path = "/image/svg", responses((status = 200, description = "Return an SVG image")))]
pub(crate) async fn serve_svg(State(_state): State<AppState>) -> Response {
    build_image_response("image/svg+xml", SVG_BYTES)
}

fn build_image_response(content_type: &'static str, bytes: &'static [u8]) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static(content_type))
        .header(header::CONTENT_LENGTH, bytes.len())
        .body(Body::from(bytes))
        .expect("static image response builder cannot fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    fn test_app() -> axum::Router {
        crate::app::build_app(AppState::new())
    }

    async fn get_bytes(uri: &str) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.expect("request failed");
        let status = response.status();
        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec();
        (status, headers, body)
    }

    #[tokio::test]
    async fn image_png_returns_png_bytes() {
        let (status, headers, body) = get_bytes("/image/png").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "image/png");
        let len: usize = headers
            .get(header::CONTENT_LENGTH)
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(len, body.len());
        assert_eq!(body, PNG_BYTES);
    }

    #[tokio::test]
    async fn image_alias_returns_png() {
        let (status, headers, body) = get_bytes("/image").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "image/png");
        assert_eq!(body, PNG_BYTES);
    }

    #[tokio::test]
    async fn image_jpeg_returns_jpeg_bytes() {
        let (status, headers, body) = get_bytes("/image/jpeg").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "image/jpeg");
        assert_eq!(body, JPEG_BYTES);
    }

    #[tokio::test]
    async fn image_webp_returns_webp_bytes() {
        let (status, headers, body) = get_bytes("/image/webp").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "image/webp");
        assert_eq!(body, WEBP_BYTES);
    }

    #[tokio::test]
    async fn image_svg_returns_svg_bytes() {
        let (status, headers, body) = get_bytes("/image/svg").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "image/svg+xml");
        let len: usize = headers
            .get(header::CONTENT_LENGTH)
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(len, body.len());
        assert_eq!(body, SVG_BYTES);
    }
}
