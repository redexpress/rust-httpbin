pub(crate) mod anything;
pub(crate) mod uuid;

use crate::state::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    Router::new().merge(uuid::route()).merge(anything::route())
}
