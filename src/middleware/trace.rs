use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::info_span;

/// Middleware that creates a tracing span for each request.
///
/// The span includes the request id (set by `request_id` middleware, which
/// runs before this) so downstream logs are correlated.
pub async fn trace_layer(req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let method = req.method().to_string();
    let uri = req.uri().to_string();

    let span = info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        uri = %uri,
    );

    let _enter = span.enter();
    next.run(req).await
}
