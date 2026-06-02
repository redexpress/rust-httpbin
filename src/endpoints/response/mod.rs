pub(crate) mod delay;
pub(crate) mod redirect;
pub(crate) mod status;
pub(crate) mod stream;

use axum::Router;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(status::route())
        .merge(delay::route())
        .merge(redirect::route())
        .merge(stream::route())
}
