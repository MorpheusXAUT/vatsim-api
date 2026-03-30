//! Async HTTP client for the VATSIM APIs.
//!
//! [`VatsimClient`] provides methods for fetching the
//! [data feed](https://vatsim.dev/api/data-api/get-network-data) and
//! [slurper](https://vatsim.dev/api/slurper-api/get-user-info) endpoints.
//! Datafeed responses are cached locally; use [`CachePolicy`] to control
//! whether a call may return a cached copy or must refresh from the network.

pub mod datafeed;
pub mod slurper;

use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

const STATUS_URL: &str = "https://status.vatsim.net/status.json";
const DATAFEED_ENDPOINT: &str = "/v3/vatsim-data.json";
const SLURPER_ENDPOINT: &str = "/users/info";
const DATAFEED_CACHE_TTL_SECS: u64 = 15;

/// Controls whether [`VatsimClient::datafeed`] may return a cached response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CachePolicy {
    /// Return a cached copy if one exists and is still within the TTL.
    Cached,
    /// Always fetch from the network, ignoring any cached data.
    Refresh,
}

/// Async HTTP client for the VATSIM data feed and slurper APIs.
///
/// The client is cheaply cloneable (backed by an [`Arc`]) and safe to share
/// across tasks. Datafeed responses are cached locally according to
/// [`ClientConfig::cache_ttl_secs`].
#[derive(Clone)]
pub struct VatsimClient {
    inner: Arc<ClientInner>,
}

/// Configuration for a [`VatsimClient`].
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// URL of the [VATSIM status endpoint](https://status.vatsim.net/status.json)
    /// used to discover data feed mirror URLs.
    pub status_url: String,
    /// If set, bypasses status URL discovery and fetches the data feed from this
    /// URL directly.
    pub datafeed_url_override: Option<String>,
    /// If set, uses this URL for slurper requests instead of the default.
    pub slurper_url_override: Option<String>,
    /// How many seconds a cached datafeed response is considered fresh.
    pub cache_ttl_secs: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            status_url: STATUS_URL.to_string(),
            datafeed_url_override: None,
            slurper_url_override: None,
            cache_ttl_secs: DATAFEED_CACHE_TTL_SECS,
        }
    }
}

struct ClientInner {
    http: Client,
    config: ClientConfig,
    datafeed_cache: RwLock<Option<(crate::types::datafeed::DataFeed, std::time::Instant)>>,
    datafeed_urls: RwLock<Vec<String>>,
}

impl VatsimClient {
    /// Creates a new [`VatsimClient`] with the default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(ClientConfig::default())
    }

    /// Creates a new [`VatsimClient`] pointing at a mock server.
    ///
    /// Both the datafeed and slurper URL overrides are set to routes under
    /// the given base URL.
    #[must_use]
    pub fn with_mock_base_url(mock_base_url: impl Into<String>) -> Self {
        let base = mock_base_url.into().trim_end_matches('/').to_owned();
        Self::with_config(ClientConfig {
            datafeed_url_override: Some(format!("{base}{DATAFEED_ENDPOINT}")),
            slurper_url_override: Some(format!("{base}{SLURPER_ENDPOINT}")),
            ..Default::default()
        })
    }

    /// Creates a new [`VatsimClient`] with the given [`ClientConfig`].
    ///
    /// # Panics
    ///
    /// Panics if the underlying HTTP client cannot be built (e.g. TLS
    /// initialization failure). This should not happen under normal
    /// circumstances.
    #[must_use]
    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            inner: Arc::new(ClientInner {
                http: Client::builder()
                    .user_agent(concat!(
                        env!("CARGO_PKG_NAME"),
                        "/",
                        env!("CARGO_PKG_VERSION"),
                        " (+",
                        env!("CARGO_PKG_REPOSITORY"),
                        ")"
                    ))
                    .build()
                    .expect("failed to build HTTP client"),
                config,
                datafeed_cache: RwLock::new(None),
                datafeed_urls: RwLock::new(Vec::new()),
            }),
        }
    }
}

impl Default for VatsimClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct StatusResponse {
    data: StatusData,
}

#[derive(Deserialize)]
struct StatusData {
    v3: Vec<String>,
}
