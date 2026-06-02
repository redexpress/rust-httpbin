use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize structured logging.
    // Set RUST_LOG=debug to see per-request spans and access logs.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Build shared application state
    let state = axum_httpbin::state::AppState::new();

    // Compose the router
    let app = axum_httpbin::app::build_app(state);

    // Bind and serve
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind to 127.0.0.1:3000");

    info!("Axum Httpbin listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("server error");
}
