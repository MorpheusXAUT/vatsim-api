// Route handlers are not user-facing Rust API; error semantics are HTTP status codes.
#![allow(clippy::missing_errors_doc)]

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::mock::state::{MockState, SharedState};
use crate::types::CertificateId;
use crate::types::datafeed::{Atis, Controller, Pilot, Prefile, Server};

type ApiResult<T> = Result<Json<T>, StatusCode>;
type StatusCodeResult = Result<StatusCode, StatusCode>;

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/api/state", get(get_state).put(put_state))
        .route("/api/reset", post(reset_state))
        .route("/api/pilots", get(list_pilots).post(upsert_pilot))
        .route(
            "/api/pilots/{cid}",
            get(get_pilot).put(put_pilot).delete(delete_pilot),
        )
        .route(
            "/api/controllers",
            get(list_controllers).post(upsert_controller),
        )
        .route(
            "/api/controllers/{cid}",
            get(get_controller)
                .put(put_controller)
                .delete(delete_controller),
        )
        .route("/api/atis", get(list_atis).post(upsert_atis))
        .route(
            "/api/atis/{callsign}",
            get(get_atis).put(put_atis).delete(delete_atis),
        )
        .route("/api/prefiles", get(list_prefiles).post(upsert_prefile))
        .route(
            "/api/prefiles/{cid}",
            get(get_prefile).put(put_prefile).delete(delete_prefile),
        )
        .route("/api/servers", get(list_servers).post(upsert_server))
        .route(
            "/api/servers/{ident}",
            get(get_server).put(put_server).delete(delete_server),
        )
}

async fn get_state(State(state): State<SharedState>) -> ApiResult<MockState> {
    let state = state.read().await;
    Ok(Json(state.clone()))
}

async fn put_state(
    State(state): State<SharedState>,
    Json(new_state): Json<MockState>,
) -> StatusCodeResult {
    let mut state = state.write().await;
    state.replace(new_state);
    Ok(StatusCode::NO_CONTENT)
}

async fn reset_state(State(state): State<SharedState>) -> StatusCodeResult {
    let mut state = state.write().await;
    state.reset();
    Ok(StatusCode::NO_CONTENT)
}

async fn list_pilots(State(state): State<SharedState>) -> ApiResult<Vec<Pilot>> {
    let state = state.read().await;
    Ok(Json(state.pilots.clone()))
}

async fn get_pilot(State(state): State<SharedState>, Path(cid): Path<u32>) -> ApiResult<Pilot> {
    let state = state.read().await;
    state
        .pilot(CertificateId::new(cid))
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn upsert_pilot(
    State(state): State<SharedState>,
    Json(pilot): Json<Pilot>,
) -> StatusCodeResult {
    let mut state = state.write().await;
    state.upsert_pilot(pilot);
    Ok(StatusCode::NO_CONTENT)
}

async fn put_pilot(
    State(state): State<SharedState>,
    Path(cid): Path<u32>,
    Json(mut pilot): Json<Pilot>,
) -> StatusCodeResult {
    pilot.cid = CertificateId::new(cid);
    let mut state = state.write().await;
    state.upsert_pilot(pilot);
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_pilot(State(state): State<SharedState>, Path(cid): Path<u32>) -> StatusCodeResult {
    let mut state = state.write().await;
    if state.remove_pilot(CertificateId::new(cid)) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_controllers(State(state): State<SharedState>) -> ApiResult<Vec<Controller>> {
    let state = state.read().await;
    Ok(Json(state.controllers.clone()))
}

async fn get_controller(
    State(state): State<SharedState>,
    Path(cid): Path<u32>,
) -> ApiResult<Controller> {
    let state = state.read().await;
    state
        .controller(CertificateId::new(cid))
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn upsert_controller(
    State(state): State<SharedState>,
    Json(controller): Json<Controller>,
) -> StatusCodeResult {
    let mut state = state.write().await;
    state.upsert_controller(controller);
    Ok(StatusCode::NO_CONTENT)
}

async fn put_controller(
    State(state): State<SharedState>,
    Path(cid): Path<u32>,
    Json(mut controller): Json<Controller>,
) -> StatusCodeResult {
    controller.cid = CertificateId::new(cid);
    let mut state = state.write().await;
    state.upsert_controller(controller);
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_controller(
    State(state): State<SharedState>,
    Path(cid): Path<u32>,
) -> StatusCodeResult {
    let mut state = state.write().await;
    if state.remove_controller(CertificateId::new(cid)) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_atis(State(state): State<SharedState>) -> ApiResult<Vec<Atis>> {
    let state = state.read().await;
    Ok(Json(state.atis.clone()))
}

async fn get_atis(
    State(state): State<SharedState>,
    Path(callsign): Path<String>,
) -> ApiResult<Atis> {
    let state = state.read().await;
    state
        .atis(&callsign)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn upsert_atis(State(state): State<SharedState>, Json(atis): Json<Atis>) -> StatusCodeResult {
    let mut state = state.write().await;
    state.upsert_atis(atis);
    Ok(StatusCode::NO_CONTENT)
}

async fn put_atis(
    State(state): State<SharedState>,
    Path(callsign): Path<String>,
    Json(mut atis): Json<Atis>,
) -> StatusCodeResult {
    atis.callsign = callsign;
    let mut state = state.write().await;
    state.upsert_atis(atis);
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_atis(
    State(state): State<SharedState>,
    Path(callsign): Path<String>,
) -> StatusCodeResult {
    let mut state = state.write().await;
    if state.remove_atis(&callsign) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_prefiles(State(state): State<SharedState>) -> ApiResult<Vec<Prefile>> {
    let state = state.read().await;
    Ok(Json(state.prefiles.clone()))
}

async fn get_prefile(State(state): State<SharedState>, Path(cid): Path<u32>) -> ApiResult<Prefile> {
    let state = state.read().await;
    state
        .prefile(CertificateId::new(cid))
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn upsert_prefile(
    State(state): State<SharedState>,
    Json(prefile): Json<Prefile>,
) -> StatusCodeResult {
    let mut state = state.write().await;
    state.upsert_prefile(prefile);
    Ok(StatusCode::NO_CONTENT)
}

async fn put_prefile(
    State(state): State<SharedState>,
    Path(cid): Path<u32>,
    Json(mut prefile): Json<Prefile>,
) -> StatusCodeResult {
    prefile.cid = CertificateId::new(cid);
    let mut state = state.write().await;
    state.upsert_prefile(prefile);
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_prefile(
    State(state): State<SharedState>,
    Path(cid): Path<u32>,
) -> StatusCodeResult {
    let mut state = state.write().await;
    if state.remove_prefile(CertificateId::new(cid)) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_servers(State(state): State<SharedState>) -> ApiResult<Vec<Server>> {
    let state = state.read().await;
    Ok(Json(state.servers.clone()))
}

async fn get_server(
    State(state): State<SharedState>,
    Path(ident): Path<String>,
) -> ApiResult<Server> {
    let state = state.read().await;
    state
        .server(&ident)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn upsert_server(
    State(state): State<SharedState>,
    Json(server): Json<Server>,
) -> StatusCodeResult {
    let mut state = state.write().await;
    state.upsert_server(server);
    Ok(StatusCode::NO_CONTENT)
}

async fn put_server(
    State(state): State<SharedState>,
    Path(ident): Path<String>,
    Json(mut server): Json<Server>,
) -> StatusCodeResult {
    server.ident = ident;
    let mut state = state.write().await;
    state.upsert_server(server);
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_server(
    State(state): State<SharedState>,
    Path(ident): Path<String>,
) -> StatusCodeResult {
    let mut state = state.write().await;
    if state.remove_server(&ident) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
