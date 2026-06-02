pub(crate) mod basic;
pub(crate) mod bearer;

use crate::state::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    Router::new().merge(basic::route()).merge(bearer::route())
}
