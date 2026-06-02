pub(crate) mod basic;
pub(crate) mod bearer;

use axum::Router;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(basic::route())
        .merge(bearer::route())
}
