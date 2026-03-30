#![cfg(feature = "client")]

use vatsim_api::{CachePolicy, CertificateId, ClientConfig, ClientError, VatsimClient};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn mock_client(server: &MockServer) -> VatsimClient {
    VatsimClient::with_mock_base_url(server.uri())
}

fn minimal_datafeed() -> serde_json::Value {
    serde_json::json!({
        "general": {
            "version": 3,
            "update_timestamp": "2025-01-15T12:00:00Z",
            "connected_clients": 0,
            "unique_users": 0
        },
        "pilots": [],
        "controllers": [],
        "atis": [],
        "servers": [],
        "prefiles": [],
        "facilities": [],
        "ratings": [],
        "pilot_ratings": [],
        "military_ratings": []
    })
}

#[tokio::test]
async fn datafeed_empty() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/vatsim-data.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_datafeed()))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let feed = client.datafeed(CachePolicy::Cached).await.unwrap();

    assert!(feed.pilots.is_empty());
    assert!(feed.controllers.is_empty());
    assert!(feed.atis.is_empty());
    assert_eq!(feed.general.version, 3);
    assert_eq!(feed.general.connected_clients, 0);
}

#[tokio::test]
async fn datafeed_with_pilot() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/vatsim-data.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "general": {
                "version": 3,
                "update_timestamp": "2025-01-15T12:00:00Z",
                "connected_clients": 1,
                "unique_users": 1
            },
            "pilots": [{
                "cid": 1234567,
                "name": "Test Pilot",
                "callsign": "AUA456",
                "server": "EU-C1",
                "pilot_rating": 0,
                "military_rating": 0,
                "latitude": 48.11028,
                "longitude": 16.56972,
                "altitude": 35000,
                "groundspeed": 450,
                "transponder": "4521",
                "heading": 270,
                "qnh_i_hg": 29.92,
                "qnh_mb": 1013,
                "flight_plan": null,
                "logon_time": "2025-01-15T10:00:00Z",
                "last_updated": "2025-01-15T12:00:00Z"
            }],
            "controllers": [],
            "atis": [],
            "servers": [],
            "prefiles": [],
            "facilities": [],
            "ratings": [],
            "pilot_ratings": [],
            "military_ratings": []
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let feed = client.datafeed(CachePolicy::Cached).await.unwrap();

    assert_eq!(feed.pilots.len(), 1);
    let pilot = &feed.pilots[0];
    assert_eq!(pilot.cid, CertificateId::new(1_234_567));
    assert_eq!(pilot.callsign, "AUA456");
    assert_eq!(pilot.altitude, 35000);
    assert_eq!(pilot.groundspeed, 450);
}

#[tokio::test]
async fn datafeed_with_controller_and_atis() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/vatsim-data.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "general": {
                "version": 3,
                "update_timestamp": "2025-01-15T12:00:00Z",
                "connected_clients": 2,
                "unique_users": 2
            },
            "pilots": [],
            "controllers": [{
                "cid": 1111111,
                "name": "Test Controller",
                "callsign": "LOWW_TWR",
                "frequency": "119.400",
                "facility": 4,
                "rating": 3,
                "server": "EU-C1",
                "visual_range": 50,
                "text_atis": ["LOWW ATIS INFO A"],
                "last_updated": "2025-01-15T12:00:00Z",
                "logon_time": "2025-01-15T08:00:00Z"
            }],
            "atis": [{
                "cid": 2222222,
                "name": "ATIS Bot",
                "callsign": "LOWW_ATIS",
                "frequency": "122.955",
                "facility": 4,
                "rating": 1,
                "server": "EU-C1",
                "visual_range": 0,
                "atis_code": "A",
                "text_atis": ["LOWW INFO A", "RWY 29"],
                "last_updated": "2025-01-15T12:00:00Z",
                "logon_time": "2025-01-15T07:00:00Z"
            }],
            "servers": [],
            "prefiles": [],
            "facilities": [],
            "ratings": [],
            "pilot_ratings": [],
            "military_ratings": []
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let feed = client.datafeed(CachePolicy::Cached).await.unwrap();

    assert_eq!(feed.controllers.len(), 1);
    assert_eq!(feed.controllers[0].callsign, "LOWW_TWR");
    assert_eq!(feed.controllers[0].frequency, "119.400");

    assert_eq!(feed.atis.len(), 1);
    assert_eq!(feed.atis[0].callsign, "LOWW_ATIS");
    assert_eq!(feed.atis[0].atis_code.as_deref(), Some("A"));
    assert_eq!(
        feed.atis[0].text_atis.as_deref(),
        Some(["LOWW INFO A".to_owned(), "RWY 29".to_owned()].as_slice())
    );
}

#[tokio::test]
async fn datafeed_cache_returns_cached_copy() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/vatsim-data.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_datafeed()))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let _ = client.datafeed(CachePolicy::Cached).await.unwrap();
    let _ = client.datafeed(CachePolicy::Cached).await.unwrap();
}

#[tokio::test]
async fn datafeed_refresh_bypasses_cache() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/vatsim-data.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_datafeed()))
        .expect(2)
        .mount(&server)
        .await;

    let client = mock_client(&server);

    let _ = client.datafeed(CachePolicy::Cached).await.unwrap();
    let _ = client.datafeed(CachePolicy::Refresh).await.unwrap();
}

#[tokio::test]
async fn datafeed_non_200_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/vatsim-data.json"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let err = client.datafeed(CachePolicy::Cached).await.unwrap_err();

    assert!(
        matches!(err, ClientError::Http(_)),
        "expected Http error, got {err:?}"
    );
}

#[tokio::test]
async fn datafeed_invalid_json_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/vatsim-data.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let err = client.datafeed(CachePolicy::Cached).await.unwrap_err();

    assert!(
        matches!(err, ClientError::Http(_)),
        "expected Http error from bad JSON, got {err:?}"
    );
}

#[tokio::test]
async fn datafeed_status_discovery() {
    let server = MockServer::start().await;

    // The status endpoint returns the datafeed URL.
    let status_json = serde_json::json!({
        "data": {
            "v3": [format!("{}/v3/vatsim-data.json", server.uri())]
        }
    });
    Mock::given(method("GET"))
        .and(path("/status.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(status_json))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v3/vatsim-data.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(minimal_datafeed()))
        .mount(&server)
        .await;

    let client = VatsimClient::with_config(ClientConfig {
        status_url: format!("{}/status.json", server.uri()),
        ..Default::default()
    });

    let feed = client.datafeed(CachePolicy::Cached).await.unwrap();
    assert_eq!(feed.general.version, 3);
}

#[tokio::test]
async fn datafeed_status_no_urls_returns_error() {
    let server = MockServer::start().await;

    let status_json = serde_json::json!({
        "data": {
            "v3": []
        }
    });
    Mock::given(method("GET"))
        .and(path("/status.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(status_json))
        .mount(&server)
        .await;

    let client = VatsimClient::with_config(ClientConfig {
        status_url: format!("{}/status.json", server.uri()),
        ..Default::default()
    });

    let err = client.datafeed(CachePolicy::Cached).await.unwrap_err();
    assert!(
        matches!(err, ClientError::NoDataUrls),
        "expected NoDataUrls, got {err:?}"
    );
}

#[tokio::test]
async fn datafeed_with_flight_plan() {
    let feed_json = serde_json::json!({
        "general": {
            "version": 3,
            "update_timestamp": "2025-01-15T12:00:00Z",
            "connected_clients": 1,
            "unique_users": 1
        },
        "pilots": [{
            "cid": 9999999,
            "name": "Flight Plan Test",
            "callsign": "AUA123",
            "server": "EU-C1",
            "pilot_rating": 0,
            "military_rating": 0,
            "latitude": 48.11028,
            "longitude": 16.56972,
            "altitude": 0,
            "groundspeed": 0,
            "transponder": "1000",
            "heading": 0,
            "qnh_i_hg": 29.92,
            "qnh_mb": 1013,
            "flight_plan": {
                "flight_rules": "I",
                "aircraft": "A320/M-SDE2E3FGHIJ4J5M1RWXY/LB1",
                "aircraft_faa": "A320/L",
                "aircraft_short": "A320",
                "departure": "LOWW",
                "arrival": "LOWS",
                "alternate": "LOWL",
                "deptime": "1200",
                "enroute_time": "0145",
                "fuel_time": "0345",
                "remarks": "PBN/A1B1D1L1O1S2 DOF/250115 REG/OE-LBA",
                "route": "SOVI1C SOVIL SITNI BAGSI MATIG MATI2R",
                "revision_id": 1,
                "assigned_transponder": "4521"
            },
            "logon_time": "2025-01-15T11:30:00Z",
            "last_updated": "2025-01-15T12:00:00Z"
        }],
        "controllers": [],
        "atis": [],
        "servers": [],
        "prefiles": [],
        "facilities": [],
        "ratings": [],
        "pilot_ratings": [],
        "military_ratings": []
    });

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/vatsim-data.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(feed_json))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let feed = client.datafeed(CachePolicy::Cached).await.unwrap();

    let pilot = &feed.pilots[0];
    let fp = pilot
        .flight_plan
        .as_ref()
        .expect("flight plan should exist");
    assert_eq!(fp.departure, "LOWW");
    assert_eq!(fp.arrival, "LOWS");
    assert_eq!(fp.alternate, "LOWL");
    assert_eq!(fp.aircraft_short, "A320");
}

#[tokio::test]
async fn slurper_offline_user() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .and(query_param("cid", "1234567"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let conns = client
        .user_connections(CertificateId::new(1_234_567))
        .await
        .unwrap();

    assert!(conns.is_empty());
}

#[tokio::test]
async fn slurper_single_pilot() {
    let body = "1234567,AUA456,pilot,,,48.11028,16.56972,0,0,0,0,\n";

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .and(query_param("cid", "1234567"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let conns = client
        .user_connections(CertificateId::new(1_234_567))
        .await
        .unwrap();

    assert_eq!(conns.len(), 1);
    assert_eq!(conns[0].callsign, "AUA456");
    assert_eq!(conns[0].frequency, None);
    assert_eq!(conns[0].visual_range, None);
}

#[tokio::test]
async fn slurper_single_controller() {
    let body = "1234567,LOWW_TWR,atc,119.400,50,48.11028,16.56972,\n";

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .and(query_param("cid", "1234567"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let conns = client
        .user_connections(CertificateId::new(1_234_567))
        .await
        .unwrap();

    assert_eq!(conns.len(), 1);
    assert_eq!(conns[0].callsign, "LOWW_TWR");
    assert_eq!(conns[0].frequency.as_deref(), Some("119.400"));
    assert_eq!(conns[0].visual_range, Some(50));
}

#[tokio::test]
async fn slurper_multiple_connections() {
    let body = "\
1234567,AUA456,pilot,,,48.11028,16.56972,0,0,0,0,\n\
1234567,LOWW_TWR,atc,119.400,50,48.11028,16.56972,\n";

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .and(query_param("cid", "1234567"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let conns = client
        .user_connections(CertificateId::new(1_234_567))
        .await
        .unwrap();

    assert_eq!(conns.len(), 2);
    assert_eq!(conns[0].callsign, "AUA456");
    assert_eq!(conns[1].callsign, "LOWW_TWR");
}

#[tokio::test]
async fn slurper_controller_with_secondary_positions() {
    let body =
        "1234567,LOVV_CTR,atc,132.600,256,47.66667,14.33333,47.26,11.34,46.99,15.44,0,0,0,0,\n";

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .and(query_param("cid", "1234567"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let conns = client
        .user_connections(CertificateId::new(1_234_567))
        .await
        .unwrap();

    assert_eq!(conns.len(), 1);
    assert_eq!(conns[0].secondary_positions.len(), 2);
    assert!((conns[0].secondary_positions[0].0 - 47.26).abs() < f64::EPSILON);
    assert!((conns[0].secondary_positions[1].1 - 15.44).abs() < f64::EPSILON);
}

#[tokio::test]
async fn slurper_non_200_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let err = client
        .user_connections(CertificateId::new(1_234_567))
        .await
        .unwrap_err();

    assert!(
        matches!(err, ClientError::Http(_)),
        "expected Http error, got {err:?}"
    );
}

#[tokio::test]
async fn slurper_404_returns_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let err = client
        .user_connections(CertificateId::new(1_234_567))
        .await
        .unwrap_err();

    assert!(matches!(err, ClientError::Http(_)));
}

#[tokio::test]
async fn slurper_malformed_csv_returns_parse_error() {
    let body = "not,enough,fields\n";

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let err = client
        .user_connections(CertificateId::new(1_234_567))
        .await
        .unwrap_err();

    assert!(
        matches!(err, ClientError::Parse(_)),
        "expected Parse error, got {err:?}"
    );
}

#[tokio::test]
async fn slurper_sends_cid_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .and(query_param("cid", "9876543"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .expect(1)
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let _ = client
        .user_connections(CertificateId::new(9_876_543))
        .await
        .unwrap();
}

#[tokio::test]
async fn slurper_blank_lines_are_skipped() {
    let body = "\n1234567,AUA456,pilot,,,48.11028,16.56972,0,0,0,0,\n\n\n";

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users/info"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let conns = client
        .user_connections(CertificateId::new(1_234_567))
        .await
        .unwrap();

    assert_eq!(conns.len(), 1);
}
