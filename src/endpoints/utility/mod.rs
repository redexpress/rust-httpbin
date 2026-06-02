pub(crate) mod anything;
pub(crate) mod uuid;

use axum::Router;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(uuid::route())
        .merge(anything::route())
}
