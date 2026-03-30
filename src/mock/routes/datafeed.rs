use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use crate::mock::state::SharedState;
use crate::types::datafeed::DataFeed;

pub fn routes() -> Router<SharedState> {
    Router::new().route("/v3/vatsim-data.json", get(get_datafeed))
}

/// `GET /v3/vatsim-data.json`
///
/// Returns a full [`DataFeed`] JSON response built from the current
/// [`MockState`](crate::mock::state::MockState).
async fn get_datafeed(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    (StatusCode::OK, Json(DataFeed::from(&*state)))
}
