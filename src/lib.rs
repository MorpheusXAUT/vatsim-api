//! Rust types, async client, and mock server for the
//! [VATSIM](https://vatsim.net/) network APIs.
//!
//! This crate provides strongly-typed bindings for the public VATSIM APIs used
//! by the flight simulation community. It currently covers three of them:
//!
//! - **[Data feed](https://vatsim.dev/api/data-api/get-network-data)** - a JSON
//!   snapshot of every pilot, controller, and ATIS station currently connected
//!   to the network.
//! - **[Slurper](https://vatsim.dev/api/slurper-api/get-user-info)** - a CSV
//!   endpoint returning a single user's active connections.
//! - **[Connect](https://vatsim.dev/api/connect-api)** - VATSIM's OAuth2
//!   provider, covered by the [types](types::connect) and the [mock] server.
//!   The [client] does not implement the OAuth2 flow.
//!
//! The data feed and slurper are unauthenticated; Connect needs a registered
//! OAuth2 client. Further APIs listed at <https://vatsim.dev/> may be added in
//! future releases.
//!
//! # Crate layout
//!
//! The crate is organized in three layers, each behind its own feature flag:
//!
//! | Layer | Feature | Description |
//! |-------|---------|-------------|
//! | [`types`] | *(always available)* | Shared enums ([`Facility`](types::Facility), [`ControllerRating`](types::ControllerRating), etc.) and per-endpoint response structs ([`types::datafeed::DataFeed`], [`types::slurper::UserConnection`], [`types::connect::ConnectUser`]). |
//! | [`client`] | `client` | Async HTTP client ([`VatsimClient`]) that fetches and caches live data. |
//! | [`mock`] | `mock` | Embeddable mock server for integration testing. |
//!
//! # Feature flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `serde` | **yes** | `Serialize`/`Deserialize` derives on all types. |
//! | `chrono` | **yes** | Timestamps use `chrono::DateTime<Utc>` instead of raw strings. |
//! | `client` | no | Enables [`VatsimClient`], an async HTTP client for the VATSIM APIs. |
//! | `mock` | no | Enables the [`mock`] server, for integration testing without touching the live network. |
//! | `mock-bin` | no | Builds `vatsim-mock`, a standalone binary of the mock server. |
//! | `tracing` | no | Emits `tracing` spans and events from the mock server. |
//!
//! # Fetching live data
//!
//! [`VatsimClient`] discovers the data feed mirrors through the VATSIM status
//! endpoint on first use, then caches both the mirror list and the feed itself.
//! Use [`CachePolicy`] to decide whether a call may be served from that cache.
//!
//! ```rust,no_run
//! # #[cfg(feature = "client")]
//! # async fn example() -> Result<(), vatsim_api::ClientError> {
//! use vatsim_api::{CachePolicy, VatsimClient};
//!
//! let client = VatsimClient::new();
//!
//! let feed = client.datafeed(CachePolicy::Cached).await?;
//! for controller in &feed.controllers {
//!     println!("{} on {}", controller.callsign, controller.frequency);
//! }
//!
//! // The slurper answers "is this user online right now", which the data feed
//! // can only tell you by scanning every entry.
//! let connections = client.user_connections(1_000_001.into()).await?;
//! println!("{} active connection(s)", connections.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Testing against a mock network
//!
//! [`MockServer`] serves the same endpoints from in-memory state, so tests can
//! put a controller online, take them offline, or walk a full OAuth2 login
//! without touching VATSIM. The server shuts down when its handle is dropped.
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "mock", feature = "client"))]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use vatsim_api::mock::MockServer;
//! use vatsim_api::types::datafeed::Controller;
//! use vatsim_api::{CachePolicy, CertificateId};
//!
//! let handle = MockServer::builder()
//!     .controllers(vec![Controller {
//!         cid: CertificateId::new(1_000_001),
//!         callsign: "LOWW_TWR".to_owned(),
//!         frequency: "121.500".to_owned(),
//!         ..Default::default()
//!     }])
//!     .spawn()
//!     .await?;
//!
//! let feed = handle.client().datafeed(CachePolicy::Refresh).await?;
//! assert_eq!(feed.controllers.len(), 1);
//! # Ok(())
//! # }
//! ```
//!
//! # Driving the mock at runtime
//!
//! Test code can reach the server's state directly through
//! [`MockServerHandle::state`], which avoids a round trip and is the easiest way
//! to simulate a controller connecting or disconnecting mid-test. The same
//! operations are also available over HTTP under `/api/`, for test suites that
//! are not written in Rust.
//!
//! ```rust,no_run
//! # #[cfg(feature = "mock")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use vatsim_api::CertificateId;
//! use vatsim_api::mock::MockServer;
//!
//! let handle = MockServer::builder().spawn().await?;
//!
//! // Take a controller offline part-way through a test.
//! handle
//!     .state()
//!     .write()
//!     .await
//!     .remove_controller(CertificateId::new(1_000_001));
//! # Ok(())
//! # }
//! ```
//!
//! # Public dependencies
//!
//! `chrono`, `serde`, `tokio` and `reqwest` appear in this crate's public API,
//! so a major version bump of any of them is a breaking change here. `chrono`
//! and `reqwest` are re-exported ([`chrono`], [`reqwest`]) so that consumers can
//! depend on exactly the versions this crate was built against.

#![warn(clippy::pedantic, missing_docs)]
// VATSIM, OAuth2, CID and similar acronyms read fine in prose without backticks.
#![allow(clippy::doc_markdown)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// The crate-level docs link into the feature-gated `client` and `mock` modules.
// docs.rs builds with all features so those links resolve there; in a
// reduced-feature build the items genuinely do not exist, which is not a defect
// worth failing the doc build over.
#![cfg_attr(
    not(all(feature = "client", feature = "mock")),
    allow(rustdoc::broken_intra_doc_links)
)]

pub mod error;
pub mod types;

#[cfg(feature = "client")]
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
pub mod client;

#[cfg(feature = "mock")]
#[cfg_attr(docsrs, doc(cfg(feature = "mock")))]
pub mod mock;

#[cfg(feature = "chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
pub use chrono;

#[cfg(feature = "client")]
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
pub use reqwest;

pub use error::ParseError;
pub use types::CertificateId;

#[cfg(feature = "client")]
pub use client::{CachePolicy, ClientConfig, VatsimClient};

#[cfg(feature = "client")]
pub use error::ClientError;

#[cfg(feature = "mock")]
pub use mock::{MockServer, MockServerBuilder, MockServerHandle};
