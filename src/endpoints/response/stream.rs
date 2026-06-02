use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::{routing::get, Router};
use futures::stream::{self, Stream, StreamExt};
use std::convert::Infallible;
use std::time::Duration;

use crate::error::AppError;
use crate::models::response::StreamItem;
use crate::state::AppState;

pub fn route() -> Router<AppState> {
    Router::new().route("/stream/{n}", get(handler))
}

/// `GET /stream/:n` — streams N JSON objects via Server-Sent Events.
///
/// Max 100 items. One item per ~200ms.
async fn handler(
    State(_state): State<AppState>,
    Path(n): Path<usize>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    if n > 100 {
        return Err(AppError::BadRequest(
            "stream count must not exceed 100".into(),
        ));
    }

    let stream = stream::iter(0..n).then(move |i| {
        let url = format!("/stream/{n}");
        async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let item = StreamItem { id: i, url };
            let json = serde_json::to_string(&item).unwrap();
            Ok::<_, Infallible>(Event::default().data(json))
        }
    });

    Ok(Sse::new(stream))
}
