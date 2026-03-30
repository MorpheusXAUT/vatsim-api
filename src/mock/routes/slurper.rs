use axum::Router;
use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;

use crate::mock::state::SharedState;
use crate::types::CertificateId;
use crate::types::slurper::{SlurperFacilityType, UserConnection};

#[derive(serde::Deserialize)]
struct SlurperQuery {
    cid: u32,
}

pub fn routes() -> Router<SharedState> {
    Router::new().route("/users/info", get(get_user_connections))
}

/// `GET /users/info?cid=...`
///
/// Returns CSV lines describing all active connections for the given CID,
/// mimicking the real [slurper API](https://vatsim.dev/api/slurper-api/get-user-info).
async fn get_user_connections(
    State(state): State<SharedState>,
    Query(query): Query<SlurperQuery>,
) -> impl IntoResponse {
    let cid = CertificateId::new(query.cid);
    let state = state.read().await;

    let mut lines = Vec::new();

    for pilot in state.pilots.iter().filter(|p| p.cid == cid) {
        let conn = UserConnection {
            cid: pilot.cid,
            callsign: pilot.callsign.clone(),
            facility_type: SlurperFacilityType::Pilot,
            frequency: None,
            visual_range: None,
            latitude: pilot.latitude,
            longitude: pilot.longitude,
            secondary_positions: Vec::new(),
        };
        lines.push(conn.to_csv_line());
    }

    for controller in state.controllers.iter().filter(|c| c.cid == cid) {
        let conn = UserConnection {
            cid: controller.cid,
            callsign: controller.callsign.clone(),
            facility_type: SlurperFacilityType::Atc,
            frequency: Some(controller.frequency.clone()),
            visual_range: Some(controller.visual_range),
            latitude: 0.0,
            longitude: 0.0,
            secondary_positions: Vec::new(),
        };
        lines.push(conn.to_csv_line());
    }

    for atis in state.atis.iter().filter(|a| a.cid == cid) {
        let conn = UserConnection {
            cid: atis.cid,
            callsign: atis.callsign.clone(),
            facility_type: SlurperFacilityType::Atc,
            frequency: Some(atis.frequency.clone()),
            visual_range: Some(atis.visual_range),
            latitude: 0.0,
            longitude: 0.0,
            secondary_positions: Vec::new(),
        };
        lines.push(conn.to_csv_line());
    }

    let body = lines.join("\n");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )
}
