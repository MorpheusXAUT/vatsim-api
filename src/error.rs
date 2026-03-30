//! Error types shared across the crate.
//!
//! [`ParseError`] is always available for type-conversion and CSV-parsing
//! failures. [`ClientError`] is gated behind the `client` feature and wraps
//! HTTP and parse errors from [`crate::client::VatsimClient`] methods.

use thiserror::Error;

/// Errors from parsing VATSIM types or CSV data.
#[derive(Debug, Error)]
pub enum ParseError {
    /// An unrecognized string or integer value for a known type (e.g. facility,
    /// rating).
    #[error("unknown {kind}: {value}")]
    UnknownValue { kind: &'static str, value: String },
    /// A malformed line in the slurper CSV response.
    #[error("invalid slurper CSV at line {line}: {reason}")]
    InvalidSlurperCsv { line: usize, reason: String },
}

/// Errors from [`VatsimClient`](crate::client::VatsimClient) operations.
#[cfg(feature = "client")]
#[derive(Debug, Error)]
pub enum ClientError {
    /// An HTTP-level failure (connection, timeout, non-success status code).
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// The response body could not be deserialized or parsed.
    #[error("failed to parse response: {0}")]
    Parse(#[from] ParseError),

    /// The [VATSIM status endpoint](https://status.vatsim.net/status.json)
    /// returned an empty list of data URLs.
    #[error("status endpoint returned no usable data URLs")]
    NoDataUrls,
}
