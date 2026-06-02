use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

use axum_httpbin::utils::raw_headers::{HeaderPeeker, InjectRawHeaders};

#[tokio::main]
async fn main() {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Build application
    let state = axum_httpbin::state::AppState::new();
    let app = axum_httpbin::app::build_app(state);

    // Bind
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind to 127.0.0.1:3000");

    info!(
        "Axum Httpbin listening on http://{}",
        listener.local_addr().unwrap()
    );

    // Manual accept loop — peeks at raw TCP bytes to preserve header casing
    loop {
        let (stream, _) = listener.accept().await.expect("accept failed");
        let app = app.clone();

        tokio::spawn(async move {
            // Peek at raw HTTP headers before hyper parses them
            let (peeker, raw_headers) = match HeaderPeeker::new(stream).peek_and_parse().await {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("failed to peek headers: {}", e);
                    return;
                }
            };

            let io = TokioIo::new(peeker);

            // Wrap router so every request on this connection sees raw headers
            let service = InjectRawHeaders::new(app.clone(), raw_headers);
            let service = TowerToHyperService::new(service);

            let mut builder = http1::Builder::new();
            // Not strictly needed now since we parse raw bytes ourselves,
            // but kept for consistency
            builder.preserve_header_case(true);

            if let Err(err) = builder.serve_connection(io, service).await {
                tracing::error!("connection error: {}", err);
            }
        });
    }
}
