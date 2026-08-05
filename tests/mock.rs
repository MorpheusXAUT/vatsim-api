#![cfg(feature = "mock")]

use vatsim_api::CachePolicy;
use vatsim_api::mock::MockServer;
use vatsim_api::mock::state::MockState;
use vatsim_api::types::connect::{
    ConnectRatingInfo, ConnectUser, ConnectUserResponse, NamedInfo, OAuthInfo, PersonalDetails,
    TokenResponse, VatsimDetails,
};
use vatsim_api::types::datafeed::{
    Atis, Controller, DataFeed, FlightPlan, FlightRules, Pilot, Prefile, Server,
};
use vatsim_api::types::{CertificateId, ControllerRating, Facility, MilitaryRating, PilotRating};

fn test_pilot(cid: u32, callsign: &str) -> Pilot {
    Pilot {
        cid: CertificateId::new(cid),
        name: "Test Pilot".to_owned(),
        callsign: callsign.to_owned(),
        server: "EUROPE-C1".to_owned(),
        pilot_rating: PilotRating::BasicMember,
        military_rating: MilitaryRating::NoMilitaryRating,
        latitude: 48.11028,
        longitude: 16.56972,
        altitude: 35000,
        groundspeed: 450,
        transponder: "4521".to_owned(),
        heading: 270,
        qnh_i_hg: 29.92,
        qnh_mb: 1013,
        flight_plan: None,
        logon_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
    }
}

fn test_controller(cid: u32, callsign: &str) -> Controller {
    Controller {
        cid: CertificateId::new(cid),
        name: "Test Controller".to_owned(),
        callsign: callsign.to_owned(),
        frequency: "119.400".to_owned(),
        facility: Facility::Tower,
        rating: ControllerRating::SeniorStudent,
        server: "EUROPE-C1".to_owned(),
        visual_range: 50,
        text_atis: None,
        logon_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
    }
}

fn test_atis(cid: u32, callsign: &str) -> Atis {
    Atis {
        cid: CertificateId::new(cid),
        name: "ATIS Bot".to_owned(),
        callsign: callsign.to_owned(),
        frequency: "118.525".to_owned(),
        facility: Facility::Observer,
        rating: ControllerRating::Observer,
        server: "EUROPE-C1".to_owned(),
        visual_range: 0,
        atis_code: Some("A".to_owned()),
        text_atis: Some(vec!["LOWW INFO A".to_owned(), "RWY 29".to_owned()]),
        logon_time: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
    }
}

fn test_server(ident: &str) -> Server {
    Server {
        ident: ident.to_owned(),
        hostname_or_ip: "127.0.0.1".to_owned(),
        location: "Vienna, Austria".to_owned(),
        name: "EUROPE-C1".to_owned(),
        client_connections_allowed: true,
        is_sweatbox: false,
    }
}

fn test_prefile(cid: u32, callsign: &str) -> Prefile {
    Prefile {
        cid: CertificateId::new(cid),
        name: "Test Pilot".to_owned(),
        callsign: callsign.to_owned(),
        flight_plan: FlightPlan {
            flight_rules: FlightRules::IFR,
            aircraft: "B738/M-SDE3FGHIM1RWXY/LB1".to_owned(),
            aircraft_faa: "B738/L".to_owned(),
            aircraft_short: "B738".to_owned(),
            departure: "LOWW".to_owned(),
            arrival: "LOWI".to_owned(),
            alternate: "LOWS".to_owned(),
            deptime: "1430".to_owned(),
            enroute_time: "0045".to_owned(),
            fuel_time: "0200".to_owned(),
            remarks: "/v/".to_owned(),
            route: "LANUX DCT RTT".to_owned(),
            revision_id: 1,
            assigned_transponder: "4521".to_owned(),
        },
        last_updated: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn spawn_empty_server() {
    let handle = MockServer::builder()
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let state = handle.state().read().await;
    assert!(state.pilots.is_empty());
    assert!(state.controllers.is_empty());
    assert!(state.atis.is_empty());
    assert!(state.servers.is_empty());
    assert!(state.prefiles.is_empty());
    drop(state);
    handle.shutdown().await;
}

#[tokio::test]
async fn builder_prepopulates_entities() {
    let pilot = test_pilot(1_000_001, "AUA100");
    let controller = test_controller(1_000_002, "LOWW_TWR");
    let atis = test_atis(1_000_003, "LOWW_ATIS");
    let server = test_server("EUROPE-C1");
    let prefile = test_prefile(1_000_004, "AUA200");

    let handle = MockServer::builder()
        .pilots(vec![pilot.clone()])
        .controllers(vec![controller.clone()])
        .atis(vec![atis.clone()])
        .servers(vec![server.clone()])
        .prefiles(vec![prefile.clone()])
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let state = handle.state().read().await;
    assert_eq!(state.pilots.len(), 1);
    assert_eq!(state.pilots[0].callsign, "AUA100");
    assert_eq!(state.controllers.len(), 1);
    assert_eq!(state.controllers[0].callsign, "LOWW_TWR");
    assert_eq!(state.atis.len(), 1);
    assert_eq!(state.atis[0].callsign, "LOWW_ATIS");
    assert_eq!(state.servers.len(), 1);
    assert_eq!(state.servers[0].ident, "EUROPE-C1");
    assert_eq!(state.prefiles.len(), 1);
    assert_eq!(state.prefiles[0].callsign, "AUA200");
}

#[tokio::test]
async fn datafeed_returns_prepopulated_data() {
    let handle = MockServer::builder()
        .pilots(vec![test_pilot(1_000_001, "AUA100")])
        .controllers(vec![test_controller(1_000_002, "LOWW_TWR")])
        .atis(vec![test_atis(1_000_003, "LOWW_ATIS")])
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let resp = reqwest::get(format!("{}/v3/vatsim-data.json", handle.base_url()))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let feed: DataFeed = resp.json().await.unwrap();
    assert_eq!(feed.general.version, 3);
    assert_eq!(feed.general.connected_clients, 3);
    assert_eq!(feed.general.unique_users, 3);
    assert_eq!(feed.pilots.len(), 1);
    assert_eq!(feed.pilots[0].callsign, "AUA100");
    assert_eq!(feed.controllers.len(), 1);
    assert_eq!(feed.atis.len(), 1);
}

#[tokio::test]
async fn datafeed_empty_server() {
    let handle = MockServer::builder()
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let feed: DataFeed = reqwest::get(format!("{}/v3/vatsim-data.json", handle.base_url()))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(feed.general.connected_clients, 0);
    assert_eq!(feed.general.unique_users, 0);
    assert!(feed.pilots.is_empty());
}

#[tokio::test]
async fn slurper_returns_pilot_connection() {
    let handle = MockServer::builder()
        .pilots(vec![test_pilot(1_000_001, "AUA100")])
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let resp = reqwest::get(format!("{}/users/info?cid=1000001", handle.base_url()))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body = resp.text().await.unwrap();
    assert!(body.contains("1000001"));
    assert!(body.contains("AUA100"));
    assert!(body.contains("pilot"));
}

#[tokio::test]
async fn slurper_returns_controller_and_atis() {
    let handle = MockServer::builder()
        .controllers(vec![test_controller(1_000_001, "LOWW_TWR")])
        .atis(vec![test_atis(1_000_001, "LOWW_ATIS")])
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let body = reqwest::get(format!("{}/users/info?cid=1000001", handle.base_url()))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let lines: Vec<&str> = body.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(body.contains("LOWW_TWR"));
    assert!(body.contains("LOWW_ATIS"));
    assert!(body.contains("atc"));
}

#[tokio::test]
async fn slurper_unknown_cid_returns_empty() {
    let handle = MockServer::builder()
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let body = reqwest::get(format!("{}/users/info?cid=9999999", handle.base_url()))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(body.is_empty());
}

#[tokio::test]
async fn api_pilot_crud() {
    let handle = MockServer::builder()
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let client = reqwest::Client::new();
    let base = handle.base_url();

    // List: empty
    let pilots: Vec<Pilot> = client
        .get(format!("{base}/api/pilots"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(pilots.is_empty());

    // Upsert
    let pilot = test_pilot(1_000_001, "AUA100");
    let resp = client
        .post(format!("{base}/api/pilots"))
        .json(&pilot)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Get by CID
    let fetched: Pilot = client
        .get(format!("{base}/api/pilots/1000001"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(fetched.callsign, "AUA100");

    // Put (update via path CID)
    let mut updated = test_pilot(9999, "AUA101");
    updated.altitude = 10000;
    let resp = client
        .put(format!("{base}/api/pilots/1000001"))
        .json(&updated)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let fetched: Pilot = client
        .get(format!("{base}/api/pilots/1000001"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(fetched.callsign, "AUA101");
    assert_eq!(fetched.altitude, 10000);
    // PUT should use the path CID, not the body CID
    assert_eq!(fetched.cid, CertificateId::new(1_000_001));

    // Delete
    let resp = client
        .delete(format!("{base}/api/pilots/1000001"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Delete again: 404
    let resp = client
        .delete(format!("{base}/api/pilots/1000001"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    // Get after delete: 404
    let resp = client
        .get(format!("{base}/api/pilots/1000001"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn api_atis_keyed_by_callsign() {
    let handle = MockServer::builder()
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let client = reqwest::Client::new();
    let base = handle.base_url();

    // Upsert two ATIS from the same CID
    let atis1 = test_atis(1_000_001, "LOWW_ATIS");
    let atis2 = test_atis(1_000_001, "LOWI_ATIS");
    client
        .post(format!("{base}/api/atis"))
        .json(&atis1)
        .send()
        .await
        .unwrap();
    client
        .post(format!("{base}/api/atis"))
        .json(&atis2)
        .send()
        .await
        .unwrap();

    // Both exist
    let all: Vec<Atis> = client
        .get(format!("{base}/api/atis"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(all.len(), 2);

    // Get by callsign
    let fetched: Atis = client
        .get(format!("{base}/api/atis/LOWW_ATIS"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(fetched.callsign, "LOWW_ATIS");

    // Delete one
    let resp = client
        .delete(format!("{base}/api/atis/LOWW_ATIS"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let all: Vec<Atis> = client
        .get(format!("{base}/api/atis"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].callsign, "LOWI_ATIS");
}

#[tokio::test]
async fn api_server_keyed_by_ident() {
    let handle = MockServer::builder()
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let client = reqwest::Client::new();
    let base = handle.base_url();

    let server = test_server("EUROPE-C1");
    client
        .post(format!("{base}/api/servers"))
        .json(&server)
        .send()
        .await
        .unwrap();

    let fetched: Server = client
        .get(format!("{base}/api/servers/EUROPE-C1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(fetched.ident, "EUROPE-C1");
    assert_eq!(fetched.location, "Vienna, Austria");

    let resp = client
        .delete(format!("{base}/api/servers/EUROPE-C1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let resp = client
        .get(format!("{base}/api/servers/EUROPE-C1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn api_put_state_replaces_all_data() {
    let handle = MockServer::builder()
        .pilots(vec![test_pilot(1_000_001, "AUA100")])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let client = reqwest::Client::new();
    let base = handle.base_url();

    // Replace with a new state containing a controller instead
    let new_state = serde_json::json!({
        "pilots": [],
        "controllers": [test_controller(2_000_001, "LOVV_CTR")],
        "atis": [],
        "servers": [],
        "prefiles": [],
        "facilities": [],
        "ratings": [],
        "pilot_ratings": [],
        "military_ratings": []
    });
    let resp = client
        .put(format!("{base}/api/state"))
        .json(&new_state)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Pilot should be gone, controller should be present
    let state: MockState = client
        .get(format!("{base}/api/state"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(state.pilots.is_empty());
    assert_eq!(state.controllers.len(), 1);
    assert_eq!(state.controllers[0].callsign, "LOVV_CTR");
}

#[tokio::test]
async fn api_reset_restores_seed() {
    let handle = MockServer::builder()
        .pilots(vec![test_pilot(1_000_001, "AUA100")])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let client = reqwest::Client::new();
    let base = handle.base_url();

    // Mutate: add a controller
    client
        .post(format!("{base}/api/controllers"))
        .json(&test_controller(2_000_001, "LOWW_TWR"))
        .send()
        .await
        .unwrap();

    // Verify mutation stuck
    let state: MockState = client
        .get(format!("{base}/api/state"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(state.pilots.len(), 1);
    assert_eq!(state.controllers.len(), 1);

    // Reset
    let resp = client
        .post(format!("{base}/api/reset"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // After reset: original pilot present, added controller gone
    let state: MockState = client
        .get(format!("{base}/api/state"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(state.pilots.len(), 1);
    assert_eq!(state.pilots[0].callsign, "AUA100");
    assert!(state.controllers.is_empty());
}

#[tokio::test]
async fn api_put_state_preserves_seed() {
    let handle = MockServer::builder()
        .pilots(vec![test_pilot(1_000_001, "AUA100")])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let client = reqwest::Client::new();
    let base = handle.base_url();

    // Replace state entirely
    let new_state = serde_json::json!({
        "pilots": [],
        "controllers": [test_controller(2_000_001, "LOVV_CTR")],
        "atis": [],
        "servers": [],
        "prefiles": [],
        "facilities": [],
        "ratings": [],
        "pilot_ratings": [],
        "military_ratings": []
    });
    client
        .put(format!("{base}/api/state"))
        .json(&new_state)
        .send()
        .await
        .unwrap();

    // Reset should still restore the original seed (the pilot), not the PUT data
    client
        .post(format!("{base}/api/reset"))
        .send()
        .await
        .unwrap();

    let state: MockState = client
        .get(format!("{base}/api/state"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(state.pilots.len(), 1);
    assert_eq!(state.pilots[0].callsign, "AUA100");
    assert!(state.controllers.is_empty());
}

#[tokio::test]
async fn api_reset_empty_server_clears_state() {
    let handle = MockServer::builder()
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let client = reqwest::Client::new();
    let base = handle.base_url();

    // Add data
    client
        .post(format!("{base}/api/pilots"))
        .json(&test_pilot(1_000_001, "AUA100"))
        .send()
        .await
        .unwrap();

    // Reset with no seed should clear everything
    client
        .post(format!("{base}/api/reset"))
        .send()
        .await
        .unwrap();

    let state: MockState = client
        .get(format!("{base}/api/state"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(state.pilots.is_empty());
}

#[tokio::test]
async fn builder_seed_from_datafeed() {
    let feed = DataFeed {
        general: vatsim_api::types::datafeed::GeneralInfo {
            version: 3,
            update_timestamp: chrono::Utc::now(),
            connected_clients: 1,
            unique_users: 1,
        },
        pilots: vec![test_pilot(1_000_001, "AUA100")],
        controllers: vec![],
        atis: vec![],
        servers: vec![],
        prefiles: vec![],
        facilities: vec![],
        ratings: vec![],
        pilot_ratings: vec![],
        military_ratings: vec![],
    };

    let handle = MockServer::builder()
        .seed(feed)
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let state = handle.state().read().await;
    assert_eq!(state.pilots.len(), 1);
    assert_eq!(state.pilots[0].callsign, "AUA100");
}

#[cfg(feature = "client")]
#[tokio::test]
async fn client_against_mock_server() {
    let handle = MockServer::builder()
        .pilots(vec![test_pilot(1_000_001, "AUA100")])
        .controllers(vec![test_controller(1_000_002, "LOWW_TWR")])
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let client = handle.client();
    let feed = client.datafeed(CachePolicy::Refresh).await.unwrap();
    assert_eq!(feed.pilots.len(), 1);
    assert_eq!(feed.pilots[0].callsign, "AUA100");
    assert_eq!(feed.controllers.len(), 1);
    assert_eq!(feed.controllers[0].callsign, "LOWW_TWR");
}

#[cfg(feature = "client")]
#[tokio::test]
async fn client_slurper_against_mock_server() {
    let handle = MockServer::builder()
        .pilots(vec![test_pilot(1_000_001, "AUA100")])
        .controllers(vec![test_controller(1_000_001, "LOWW_TWR")])
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let client = handle.client();
    let connections = client
        .user_connections(CertificateId::new(1_000_001))
        .await
        .unwrap();
    assert_eq!(connections.len(), 2);

    let callsigns: Vec<&str> = connections.iter().map(|c| c.callsign.as_str()).collect();
    assert!(callsigns.contains(&"AUA100"));
    assert!(callsigns.contains(&"LOWW_TWR"));
}

#[tokio::test]
async fn mock_state_to_datafeed_roundtrip() {
    let mut state = MockState::default();
    state.pilots = vec![test_pilot(1_000_001, "AUA100")];
    state.controllers = vec![test_controller(1_000_002, "LOWW_TWR")];

    let feed = DataFeed::from(&state);
    assert_eq!(feed.general.version, 3);
    assert_eq!(feed.general.connected_clients, 2);
    assert_eq!(feed.general.unique_users, 2);
    assert_eq!(feed.pilots.len(), 1);
    assert_eq!(feed.controllers.len(), 1);

    let state2 = MockState::from(feed);
    assert_eq!(state2.pilots.len(), 1);
    assert_eq!(state2.pilots[0].callsign, "AUA100");
    assert_eq!(state2.controllers.len(), 1);
    assert_eq!(state2.controllers[0].callsign, "LOWW_TWR");
}

#[tokio::test]
async fn general_info_counts_unique_users() {
    let mut state = MockState::default();
    // Same CID on pilot and controller - should count as 1 unique user
    state.pilots = vec![test_pilot(1_000_001, "AUA100")];
    state.controllers = vec![test_controller(1_000_001, "LOWW_TWR")];

    let info = state.general_info();
    assert_eq!(info.connected_clients, 2);
    assert_eq!(info.unique_users, 1);
}

fn test_user(cid: u32, first: &str, last: &str) -> ConnectUser {
    ConnectUser {
        cid: CertificateId::new(cid),
        personal: PersonalDetails {
            name_first: first.to_owned(),
            name_last: last.to_owned(),
            name_full: format!("{first} {last}"),
            email: Some(format!(
                "{}.{}@example.com",
                first.to_lowercase(),
                last.to_lowercase()
            )),
            country: None,
        },
        vatsim: VatsimDetails {
            rating: ConnectRatingInfo {
                id: 5,
                short: "C1".to_owned(),
                long: "Enroute Controller".to_owned(),
            },
            pilotrating: ConnectRatingInfo {
                id: 0,
                short: "NEW".to_owned(),
                long: "Basic Member".to_owned(),
            },
            region: NamedInfo {
                id: Some("EMEA".to_owned()),
                name: Some("Europe, Middle East and Africa".to_owned()),
            },
            division: NamedInfo {
                id: Some("EUD".to_owned()),
                name: Some("Europe (except UK)".to_owned()),
            },
            subdivision: None,
        },
        oauth: OAuthInfo {
            token_valid: "true".to_owned(),
        },
    }
}

#[tokio::test]
async fn oauth_full_flow() {
    let user = test_user(1_234_567, "Kennedy", "Steve");
    let handle = MockServer::builder()
        .users(vec![user.clone()])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let base = handle.base_url();
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Step 1: Authorize - should redirect with code and state
    let resp = http
        .get(format!("{base}/oauth/authorize"))
        .query(&[
            ("response_type", "code"),
            ("client_id", "123"),
            ("redirect_uri", "https://example.com/callback"),
            ("state", "csrf_token_123"),
        ])
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_redirection(),
        "expected 302, got {}",
        resp.status()
    );
    let location = resp.headers().get("location").unwrap().to_str().unwrap();
    assert!(location.starts_with("https://example.com/callback?"));
    assert!(location.contains("state=csrf_token_123"));

    // Extract the code from the redirect URL
    let url = reqwest::Url::parse(location).unwrap();
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .to_string();
    assert!(!code.is_empty());

    // Step 2: Exchange code for token
    let token_resp: TokenResponse = http
        .post(format!("{base}/oauth/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", "123"),
            ("client_secret", "secret"),
            ("redirect_uri", "https://example.com/callback"),
            ("code", &code),
        ])
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(token_resp.token_type, "Bearer");
    assert_eq!(token_resp.expires_in, 604_800);
    assert!(!token_resp.access_token.is_empty());
    assert!(!token_resp.refresh_token.is_empty());

    // Step 3: Get user details with the access token
    let user_resp: ConnectUserResponse = http
        .get(format!("{base}/api/user"))
        .bearer_auth(&token_resp.access_token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(user_resp.data.cid, CertificateId::new(1_234_567));
    assert_eq!(user_resp.data.personal.name_first, "Kennedy");
    assert_eq!(user_resp.data.personal.name_last, "Steve");
    assert_eq!(user_resp.data.vatsim.rating.id, 5);

    handle.shutdown().await;
}

#[tokio::test]
async fn oauth_authorize_no_users_returns_error() {
    let handle = MockServer::builder()
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let resp = http
        .get(format!("{}/oauth/authorize", handle.base_url()))
        .query(&[
            ("response_type", "code"),
            ("client_id", "123"),
            ("redirect_uri", "https://example.com/callback"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    handle.shutdown().await;
}

#[tokio::test]
async fn oauth_token_invalid_code_returns_error() {
    let handle = MockServer::builder()
        .users(vec![test_user(1_000_001, "Test", "User")])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let http = reqwest::Client::new();

    let resp = http
        .post(format!("{}/oauth/token", handle.base_url()))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", "123"),
            ("client_secret", "secret"),
            ("code", "invalid_code"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    handle.shutdown().await;
}

#[tokio::test]
async fn oauth_user_no_token_returns_unauthorized() {
    let handle = MockServer::builder()
        .users(vec![test_user(1_000_001, "Test", "User")])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let http = reqwest::Client::new();

    let resp = http
        .get(format!("{}/api/user", handle.base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    handle.shutdown().await;
}

#[tokio::test]
async fn oauth_user_invalid_token_returns_unauthorized() {
    let handle = MockServer::builder()
        .users(vec![test_user(1_000_001, "Test", "User")])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let http = reqwest::Client::new();

    let resp = http
        .get(format!("{}/api/user", handle.base_url()))
        .bearer_auth("invalid_token")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    handle.shutdown().await;
}

#[tokio::test]
async fn oauth_code_cannot_be_reused() {
    let handle = MockServer::builder()
        .users(vec![test_user(1_000_001, "Test", "User")])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let base = handle.base_url();
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Get an auth code
    let resp = http
        .get(format!("{base}/oauth/authorize"))
        .query(&[
            ("response_type", "code"),
            ("client_id", "123"),
            ("redirect_uri", "https://example.com/cb"),
        ])
        .send()
        .await
        .unwrap();
    let location = resp.headers().get("location").unwrap().to_str().unwrap();
    let url = reqwest::Url::parse(location).unwrap();
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .to_string();

    // First use succeeds
    let resp = http
        .post(format!("{base}/oauth/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", "123"),
            ("client_secret", "s"),
            ("code", &code),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Second use fails
    let resp = http
        .post(format!("{base}/oauth/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", "123"),
            ("client_secret", "s"),
            ("code", &code),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    handle.shutdown().await;
}

#[tokio::test]
async fn oauth_login_hint_selects_user() {
    let user_a = test_user(1_000_001, "Alpha", "User");
    let user_b = test_user(1_000_002, "Bravo", "User");
    let handle = MockServer::builder()
        .users(vec![user_a, user_b])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let base = handle.base_url();
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let resp = http
        .get(format!("{base}/oauth/authorize"))
        .query(&[
            ("response_type", "code"),
            ("client_id", "123"),
            ("redirect_uri", "https://example.com/cb"),
            ("login_hint", "1000002"),
        ])
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_redirection());
    let location = resp.headers().get("location").unwrap().to_str().unwrap();
    let url = reqwest::Url::parse(location).unwrap();
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .to_string();

    let token_resp: TokenResponse = http
        .post(format!("{base}/oauth/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", "123"),
            ("client_secret", "s"),
            ("code", &code),
        ])
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();

    let user_resp: ConnectUserResponse = http
        .get(format!("{base}/api/user"))
        .bearer_auth(&token_resp.access_token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(user_resp.data.cid, CertificateId::new(1_000_002));
    assert_eq!(user_resp.data.personal.name_first, "Bravo");

    handle.shutdown().await;
}

#[tokio::test]
async fn oauth_login_hint_unknown_cid_returns_error() {
    let handle = MockServer::builder()
        .users(vec![test_user(1_000_001, "Test", "User")])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let resp = http
        .get(format!("{}/oauth/authorize", handle.base_url()))
        .query(&[
            ("response_type", "code"),
            ("client_id", "123"),
            ("redirect_uri", "https://example.com/cb"),
            ("login_hint", "9999999"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    handle.shutdown().await;
}

#[tokio::test]
async fn api_user_crud() {
    let user = test_user(1_000_001, "Test", "User");
    let handle = MockServer::builder()
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let base = handle.base_url();
    let http = reqwest::Client::new();

    // Empty initially
    let users: Vec<ConnectUser> = http
        .get(format!("{base}/api/users"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(users.is_empty());

    // Create via POST
    http.post(format!("{base}/api/users"))
        .json(&user)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Read back
    let fetched: ConnectUser = http
        .get(format!("{base}/api/users/1000001"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(fetched.cid, CertificateId::new(1_000_001));
    assert_eq!(fetched.personal.name_first, "Test");

    // Delete
    http.delete(format!("{base}/api/users/1000001"))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Verify gone
    let resp = http
        .get(format!("{base}/api/users/1000001"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    handle.shutdown().await;
}

#[tokio::test]
async fn reset_clears_oauth_tokens() {
    let handle = MockServer::builder()
        .users(vec![test_user(1_000_001, "Test", "User")])
        .spawn()
        .await
        .expect("failed to spawn mock server");
    let base = handle.base_url();
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Complete an OAuth flow to get a token
    let resp = http
        .get(format!("{base}/oauth/authorize"))
        .query(&[
            ("response_type", "code"),
            ("client_id", "123"),
            ("redirect_uri", "https://example.com/cb"),
        ])
        .send()
        .await
        .unwrap();
    let location = resp.headers().get("location").unwrap().to_str().unwrap();
    let url = reqwest::Url::parse(location).unwrap();
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .to_string();

    let token_resp: TokenResponse = http
        .post(format!("{base}/oauth/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", "123"),
            ("client_secret", "s"),
            ("code", &code),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Token works before reset
    let resp = http
        .get(format!("{base}/api/user"))
        .bearer_auth(&token_resp.access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Reset
    http.post(format!("{base}/api/reset"))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Token no longer works after reset
    let resp = http
        .get(format!("{base}/api/user"))
        .bearer_auth(&token_resp.access_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    handle.shutdown().await;
}

/// The live VATSIM data feed reports facility and rating as numeric IDs, while
/// this crate's enums serialize to their short string codes. The mock has to
/// match the live API, or consumers parsing its JSON with anything other than
/// this crate's types see a shape production never produces.
#[tokio::test]
async fn datafeed_renders_facility_and_rating_as_numeric_ids() {
    let mut pilot = test_pilot(1_000_001, "AUA100");
    pilot.pilot_rating = PilotRating::CommercialMultiEngineLicense;
    pilot.military_rating = MilitaryRating::MilitaryInstrumentRating;

    let handle = MockServer::builder()
        .pilots(vec![pilot])
        .controllers(vec![test_controller(1_000_002, "LOWW_TWR")])
        .atis(vec![test_atis(1_000_003, "LOWW_ATIS")])
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let raw: serde_json::Value = reqwest::get(format!("{}/v3/vatsim-data.json", handle.base_url()))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Facility::Tower, ControllerRating::SeniorStudent
    assert_eq!(raw["controllers"][0]["facility"], 4);
    assert_eq!(raw["controllers"][0]["rating"], 4);

    // Facility::Observer, ControllerRating::Observer
    assert_eq!(raw["atis"][0]["facility"], 0);
    assert_eq!(raw["atis"][0]["rating"], 1);

    // Pilot ratings are a bit field, so these are 7 and 3 rather than 3 and 2.
    assert_eq!(raw["pilots"][0]["pilot_rating"], 7);
    assert_eq!(raw["pilots"][0]["military_rating"], 3);

    // Deserialization accepts the numeric form, so the typed round trip still
    // yields the enums it started with.
    let feed: DataFeed = serde_json::from_value(raw).unwrap();
    assert_eq!(feed.controllers[0].facility, Facility::Tower);
    assert_eq!(feed.controllers[0].rating, ControllerRating::SeniorStudent);
    assert_eq!(
        feed.pilots[0].pilot_rating,
        PilotRating::CommercialMultiEngineLicense
    );
}

/// The four facility types the data feed has no ID for collapse onto their
/// closest equivalent, so the mock never emits a facility a real parser would
/// reject.
#[tokio::test]
async fn datafeed_maps_facilities_without_an_id_onto_their_closest_equivalent() {
    let mut ramp = test_controller(1_000_001, "LOWW_RMP");
    ramp.facility = Facility::Ramp;
    let mut departure = test_controller(1_000_002, "LOWW_DEP");
    departure.facility = Facility::Departure;
    let mut radio = test_controller(1_000_003, "LOWW_RDO");
    radio.facility = Facility::Radio;
    let mut flow = test_controller(1_000_004, "LOWW_FMP");
    flow.facility = Facility::TrafficFlow;

    let handle = MockServer::builder()
        .controllers(vec![ramp, departure, radio, flow])
        .spawn()
        .await
        .expect("failed to spawn mock server");

    let raw: serde_json::Value = reqwest::get(format!("{}/v3/vatsim-data.json", handle.base_url()))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(raw["controllers"][0]["facility"], 3); // Ramp -> Ground
    assert_eq!(raw["controllers"][1]["facility"], 5); // Departure -> Approach
    assert_eq!(raw["controllers"][2]["facility"], 1); // Radio -> FlightServiceStation
    assert_eq!(raw["controllers"][3]["facility"], 0); // TrafficFlow -> Observer
}
