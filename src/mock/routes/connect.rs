use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::{Query, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use serde::Deserialize;
use url::Url;

use crate::mock::state::SharedState;
use crate::types::connect::{ConnectUserResponse, OAuthError, TokenResponse};

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/oauth/authorize", get(authorize))
        .route("/oauth/token", post(token))
        .route("/api/user", get(get_user))
}

#[derive(Debug, Deserialize)]
struct AuthorizeParams {
    #[allow(dead_code)]
    response_type: Option<String>,
    #[allow(dead_code)]
    client_id: Option<String>,
    redirect_uri: String,
    #[allow(dead_code)]
    scope: Option<String>,
    state: Option<String>,
}

/// `GET /oauth/authorize`
///
/// The mock auto-approves all authorization requests. It picks the first
/// available user, generates a random authorization code, stores it, and
/// redirects back to the `redirect_uri` with `code` and `state` parameters.
async fn authorize(
    State(shared): State<SharedState>,
    Query(params): Query<AuthorizeParams>,
) -> Result<Redirect, (StatusCode, Json<OAuthError>)> {
    let mut state = shared.write().await;

    let cid = state.users.first().map(|u| u.cid).ok_or_else(|| {
        oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "No users configured in mock server",
        )
    })?;

    let code = generate_mock_token("code");
    state.auth_codes.insert(code.clone(), cid);

    let mut redirect_url = Url::parse(&params.redirect_uri).map_err(|e| {
        oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            &format!("Invalid redirect_uri: {e}"),
        )
    })?;
    redirect_url.query_pairs_mut().append_pair("code", &code);
    if let Some(csrf_state) = &params.state {
        redirect_url
            .query_pairs_mut()
            .append_pair("state", csrf_state);
    }

    Ok(Redirect::to(redirect_url.as_str()))
}

#[derive(Debug, Deserialize)]
struct TokenParams {
    #[allow(dead_code)]
    grant_type: Option<String>,
    #[allow(dead_code)]
    client_id: Option<String>,
    #[allow(dead_code)]
    client_secret: Option<String>,
    #[allow(dead_code)]
    redirect_uri: Option<String>,
    code: Option<String>,
}

/// `POST /oauth/token`
///
/// Exchanges an authorization code for an access token. The mock does not
/// validate `client_id`/`client_secret`; it only verifies that `code`
/// matches a pending authorization.
async fn token(
    State(shared): State<SharedState>,
    Form(params): Form<TokenParams>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OAuthError>)> {
    let mut state = shared.write().await;

    let code = params.code.ok_or_else(|| {
        oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Missing `code` parameter",
        )
    })?;

    let cid = state.auth_codes.remove(&code).ok_or_else(|| {
        oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "Authorization code is invalid or has already been used",
        )
    })?;

    let access_token = generate_mock_token("access");
    let refresh_token = generate_mock_token("refresh");
    state.access_tokens.insert(access_token.clone(), cid);

    Ok(Json(TokenResponse {
        token_type: "Bearer".to_owned(),
        expires_in: 604_800,
        access_token,
        refresh_token,
        scopes: vec![
            "full_name".to_owned(),
            "email".to_owned(),
            "vatsim_details".to_owned(),
            "country".to_owned(),
        ],
    }))
}

/// `GET /api/user`
///
/// Returns the authenticated user's details. Requires a valid
/// `Authorization: Bearer <token>` header.
async fn get_user(
    State(shared): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<ConnectUserResponse>, (StatusCode, Json<OAuthError>)> {
    let token = extract_bearer_token(&headers).ok_or_else(|| {
        oauth_error(
            StatusCode::UNAUTHORIZED,
            "invalid_request",
            "Missing or malformed Authorization header",
        )
    })?;

    let state = shared.read().await;

    let cid = state.access_tokens.get(token).copied().ok_or_else(|| {
        oauth_error(
            StatusCode::UNAUTHORIZED,
            "invalid_grant",
            "Access token is invalid or expired",
        )
    })?;

    let user = state.user(cid).cloned().ok_or_else(|| {
        oauth_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            &format!("User with CID {cid} not found in mock state"),
        )
    })?;

    Ok(Json(ConnectUserResponse { data: user }))
}

fn oauth_error(
    status: StatusCode,
    error: &str,
    description: &str,
) -> (StatusCode, Json<OAuthError>) {
    (
        status,
        Json(OAuthError {
            error: error.to_owned(),
            error_description: Some(description.to_owned()),
            message: None,
            hint: None,
        }),
    )
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Generates a unique, opaque mock token with a descriptive prefix.
///
/// Not cryptographically secure - tokens are monotonically increasing
/// to guarantee uniqueness across concurrent requests.
fn generate_mock_token(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("mock_{prefix}_{seq:016x}")
}
