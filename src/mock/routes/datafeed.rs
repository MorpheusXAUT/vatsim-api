use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

use crate::mock::state::SharedState;
use crate::types::datafeed::DataFeed;
use crate::types::{ControllerRating, Facility, MilitaryRating, PilotRating};

pub fn routes() -> Router<SharedState> {
    Router::new().route("/v3/vatsim-data.json", get(get_datafeed))
}

/// `GET /v3/vatsim-data.json`
///
/// Returns a full [`DataFeed`] JSON response built from the current
/// [`MockState`](crate::mock::state::MockState).
///
/// Facility and rating fields are rendered as the numeric IDs the live VATSIM
/// data feed uses, rather than the short string codes this crate's enums
/// serialize to by default. See [`as_api_json`].
async fn get_datafeed(State(state): State<SharedState>) -> impl IntoResponse {
    let feed = {
        let state = state.read().await;
        DataFeed::from(&*state)
    };

    (StatusCode::OK, Json(as_api_json(&feed)))
}

/// Renders a [`DataFeed`] the way the live VATSIM API does.
///
/// This crate's [`Facility`], [`ControllerRating`], [`PilotRating`] and
/// [`MilitaryRating`] serialize to their short string codes, because those are
/// lossless and are what the VACS dataset files use. The live data feed instead
/// emits numeric IDs, and a mock that does not match it is not much of a mock,
/// so the affected fields are converted on the way out.
///
/// Deserialization accepts both representations, so this only affects consumers
/// reading the mock's JSON with something other than this crate's types.
fn as_api_json(feed: &DataFeed) -> Value {
    let mut value = serde_json::to_value(feed).unwrap_or(Value::Null);

    for (collection, fields) in [
        ("controllers", &["facility", "rating"][..]),
        ("atis", &["facility", "rating"][..]),
        ("pilots", &["pilot_rating", "military_rating"][..]),
    ] {
        let Some(entries) = value.get_mut(collection).and_then(Value::as_array_mut) else {
            continue;
        };

        for entry in entries {
            for field in fields {
                let Some(slot) = entry.get_mut(*field) else {
                    continue;
                };
                let Some(code) = slot.as_str() else { continue };

                if let Some(id) = id_for(field, code) {
                    *slot = Value::from(id);
                }
            }
        }
    }

    value
}

/// Maps a short string code back to its data feed ID, for one known field.
///
/// Returns [`None`] for an unrecognized code, which leaves the string in place
/// rather than inventing an ID.
fn id_for(field: &str, code: &str) -> Option<i8> {
    match field {
        "facility" => code.parse::<Facility>().ok().map(Facility::as_id),
        "rating" => code
            .parse::<ControllerRating>()
            .ok()
            .map(ControllerRating::as_id),
        "pilot_rating" => code.parse::<PilotRating>().ok().map(PilotRating::as_id),
        "military_rating" => code
            .parse::<MilitaryRating>()
            .ok()
            .map(MilitaryRating::as_id),
        _ => None,
    }
}
