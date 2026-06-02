pub(crate) mod headers;
pub(crate) mod ip;
pub(crate) mod user_agent;

use crate::state::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(headers::route())
        .merge(ip::route())
        .merge(user_agent::route())
}
