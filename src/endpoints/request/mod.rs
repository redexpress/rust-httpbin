pub(crate) mod delete;
pub(crate) mod get;
pub(crate) mod patch;
pub(crate) mod post;
pub(crate) mod put;

use crate::state::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(get::route())
        .merge(post::route())
        .merge(put::route())
        .merge(patch::route())
        .merge(delete::route())
}
