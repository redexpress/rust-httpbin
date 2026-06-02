use axum::Router;

use crate::endpoints;
use crate::middleware::{access_log, request_id, trace};
use crate::state::AppState;

/// Build the complete Axum application router.
///
/// This is the composition root — every route and middleware
/// is registered here.
pub fn build_app(state: AppState) -> Router {
    // Grouped route modules
    let request_routes = endpoints::request::routes();
    let inspect_routes = endpoints::inspect::routes();
    let response_routes = endpoints::response::routes();
    let auth_routes = endpoints::auth::routes();
    let utility_routes = endpoints::utility::routes();

    Router::new()
        // Endpoint groups
        .merge(request_routes)
        .merge(inspect_routes)
        .merge(response_routes)
        .merge(auth_routes)
        .merge(utility_routes)
        // Middleware (layered outermost → runs first)
        .layer(axum::middleware::from_fn(trace::trace_layer))
        .layer(axum::middleware::from_fn(request_id::request_id_layer))
        .layer(axum::middleware::from_fn(access_log::access_log_layer))
        // Shared state
        .with_state(state)
}
