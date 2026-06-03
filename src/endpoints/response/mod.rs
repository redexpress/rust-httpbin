pub(crate) mod delay;
pub(crate) mod redirect;
pub(crate) mod response_headers;
pub(crate) mod status;
pub(crate) mod stream;

use crate::state::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(status::route())
        .merge(delay::route())
        .merge(redirect::route())
        .merge(stream::route())
        .merge(response_headers::route())
}
