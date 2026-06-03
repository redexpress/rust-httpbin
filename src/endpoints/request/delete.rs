use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::{routing::delete, Router};
use std::collections::HashMap;

use crate::endpoints::request::post::build_request_info;
use crate::state::AppState;
use crate::utils::request_body::RequestBody;
use crate::utils::response_utils::ok_json;

pub fn route() -> Router<AppState> {
    Router::new().route("/delete", delete(handler))
}

/// `DELETE /delete` — echoes the incoming DELETE request.
#[utoipa::path(delete, path = "/delete", request_body = String, responses((status = 200, description = "Echo the DELETE request")))]
pub(crate) async fn handler(
    State(state): State<AppState>,
    uri: axum::http::Uri,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: RequestBody,
) -> axum::response::Response {
    let info = build_request_info(&uri, &headers, &query, &body);
    let _ = state;
    ok_json(&info)
}
