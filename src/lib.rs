//! Rust types, async client, and mock server for the
//! [VATSIM](https://vatsim.net/) network APIs.
//!
//! This crate provides strongly-typed bindings for the public VATSIM APIs used
//! by the flight simulation community. It currently covers two endpoints:
//!
//! - **[Data feed](https://vatsim.dev/api/data-api/get-network-data)** --
//!   a JSON snapshot of all pilots, controllers, and ATIS stations currently
//!   connected to the network.
//! - **[Slurper](https://vatsim.dev/api/slurper-api/get-user-info)** --
//!   a CSV endpoint that returns a user's recent connection history.
//!
//! Both endpoints are public and unauthenticated. Additional VATSIM APIs listed
//! at <https://vatsim.dev/> may be added in future releases.
//!
//! # Crate layout
//!
//! The crate is organized in three layers, each behind its own feature flag:
//!
//! | Layer | Feature | Description |
//! |-------|---------|-------------|
//! | [`types`] | *(always available)* | Shared enums ([`Facility`](types::Facility), [`ControllerRating`](types::ControllerRating), etc.) and per-endpoint response structs ([`types::datafeed::DataFeed`], [`types::slurper::UserConnection`]). |
//! | [`client`] | `client` | Async HTTP client ([`VatsimClient`]) that fetches and caches live data. |
//! | [`mock`] | `mock` | Embeddable mock server for integration testing. |
//!
//! # Feature flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `serde` | **yes** | [`Serialize`](serde::Serialize)/[`Deserialize`](serde::Deserialize) derives on all types. |
//! | `chrono` | **yes** | Timestamps use [`chrono::DateTime<Utc>`] instead of raw strings. |
//! | `client` | no | Enables [`VatsimClient`], providing a convenient async HTTP client for interacting with the VATSIM APIs. |
//! | `mock` | no | Enables the [`mock`] server, allowing integration testing without hitting the live VATSIM network. |
//! | `tracing` | no | Adds `tracing` instrumentation to the client and mock server. |

#![warn(clippy::pedantic)]

pub mod error;
pub mod types;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "chrono")]
pub use chrono;

#[cfg(feature = "client")]
pub use reqwest;

pub use error::ParseError;
pub use types::CertificateId;

#[cfg(feature = "client")]
pub use client::{CachePolicy, ClientConfig, VatsimClient};

#[cfg(feature = "client")]
pub use error::ClientError;
