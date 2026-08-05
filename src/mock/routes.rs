//! Route definitions for the mock VATSIM server.

mod api;
mod connect;
mod datafeed;
mod slurper;

use axum::Router;
use axum::http::HeaderValue;
use axum::http::header::{REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS};
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;

use super::state::SharedState;

/// Builds the complete [`axum::Router`] for the mock server.
///
/// The router exposes:
/// - VATSIM-compatible endpoints (`/v3/vatsim-data.json`, `/users/info`)
/// - Connect OAuth endpoints (`/oauth/authorize`, `/oauth/token`, `/api/user`)
/// - Management CRUD API under `/api/`
///
/// When `security_headers` is `true`, all responses include permissive CORS
/// headers and standard security headers (`X-Content-Type-Options`,
/// `X-Frame-Options`, `Referrer-Policy`).
///
/// Deliberately crate-private so that `axum` stays an implementation detail
/// rather than part of this crate's semver contract. Callers go through
/// [`MockServer`](super::MockServer).
pub(crate) fn router(state: SharedState, security_headers: bool) -> Router {
    let router = Router::new()
        .merge(datafeed::routes())
        .merge(slurper::routes())
        .merge(connect::routes())
        .merge(api::routes());

    #[cfg(feature = "tracing")]
    let router = router.layer(tower_http::trace::TraceLayer::new_for_http());

    let router = if security_headers {
        router
            .layer(CorsLayer::permissive())
            .layer(SetResponseHeaderLayer::overriding(
                X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                X_FRAME_OPTIONS,
                HeaderValue::from_static("SAMEORIGIN"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                REFERRER_POLICY,
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            ))
    } else {
        router
    };

    router.with_state(state)
}
