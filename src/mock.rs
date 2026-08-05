//! Embeddable mock server for integration testing and standalone deployment.
//!
//! The mock server serves VATSIM-compatible endpoints backed by in-memory
//! state, plus a management CRUD API for manipulating that state at runtime.
//!
//! # Quick start (tests)
//!
//! ```rust,no_run
//! # async fn example() -> std::io::Result<()> {
//! use vatsim_api::mock::MockServer;
//!
//! let handle = MockServer::builder().spawn().await?;
//! let client = handle.client();
//!
//! // Use `client` to hit the mock datafeed, slurper, etc.
//! // The server shuts down when `handle` is dropped.
//! # Ok(())
//! # }
//! ```
//!
//! # Quick start (standalone)
//!
//! ```rust,no_run
//! # async fn example() -> std::io::Result<()> {
//! use vatsim_api::mock::MockServer;
//!
//! MockServer::builder()
//!     .bind("0.0.0.0:8080")
//!     .build()
//!     .await?
//!     .serve()
//!     .await?;
//! # Ok(())
//! # }
//! ```

pub(crate) mod routes;
pub mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use state::{MockState, SharedState};

use crate::types::datafeed::DataFeed;

#[cfg(feature = "client")]
use crate::client::VatsimClient;

/// A configured mock server ready to be started.
///
/// Created via [`MockServer::builder`]. Call [`serve`](MockServer::serve) to
/// run as a blocking service, or [`spawn`](MockServer::spawn) to run in the
/// background and get a [`MockServerHandle`] back.
pub struct MockServer {
    state: SharedState,
    listener: TcpListener,
    security_headers: bool,
}

/// Builder for configuring a [`MockServer`] before starting it.
pub struct MockServerBuilder {
    state: MockState,
    bind_addr: String,
    security_headers: bool,
}

/// Handle to a running mock server spawned in the background.
///
/// Dropping the handle triggers a graceful shutdown. You can also call
/// [`shutdown`](MockServerHandle::shutdown) explicitly.
pub struct MockServerHandle {
    base_url: String,
    state: SharedState,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    join_handle: Option<JoinHandle<()>>,
}

impl MockServer {
    /// Returns a new [`MockServerBuilder`] with default settings.
    #[must_use]
    pub fn builder() -> MockServerBuilder {
        MockServerBuilder {
            state: MockState::default(),
            bind_addr: "127.0.0.1:0".to_owned(),
            security_headers: false,
        }
    }

    /// Runs the server, blocking until it is shut down.
    ///
    /// # Errors
    ///
    /// Returns an error if the server encounters a fatal I/O problem.
    pub async fn serve(self) -> std::io::Result<()> {
        let router = routes::router(self.state, self.security_headers);
        axum::serve(self.listener, router).await
    }

    /// Runs the server until `shutdown` resolves, then shuts down gracefully.
    ///
    /// # Errors
    ///
    /// Returns an error if the server encounters a fatal I/O problem.
    pub async fn serve_with_shutdown<F>(self, shutdown: F) -> std::io::Result<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let router = routes::router(self.state, self.security_headers);
        axum::serve(self.listener, router)
            .with_graceful_shutdown(shutdown)
            .await
    }

    /// Returns the socket address the server is bound to.
    ///
    /// # Errors
    ///
    /// Returns an error if the local address cannot be determined.
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Spawns the server on a background tokio task and returns a handle.
    ///
    /// The server shuts down gracefully when the handle is dropped or when
    /// [`MockServerHandle::shutdown`] is called.
    ///
    /// # Errors
    ///
    /// Returns an error if the listener's local address cannot be determined.
    pub fn spawn(self) -> std::io::Result<MockServerHandle> {
        let addr = self.listener.local_addr()?;
        let base_url = format!("http://{addr}");
        let state = Arc::clone(&self.state);

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

        let router = routes::router(self.state, self.security_headers);
        let join_handle = tokio::spawn(async move {
            axum::serve(self.listener, router)
                .with_graceful_shutdown(async move {
                    shutdown_rx.changed().await.ok();
                })
                .await
                .ok();
        });

        Ok(MockServerHandle {
            base_url,
            state,
            shutdown_tx,
            join_handle: Some(join_handle),
        })
    }
}

impl MockServerBuilder {
    /// Sets the address to bind to (default: `127.0.0.1:0` for a random port).
    #[must_use]
    pub fn bind(mut self, addr: impl Into<String>) -> Self {
        self.bind_addr = addr.into();
        self
    }

    /// Sets the initial state.
    #[must_use]
    pub fn state(mut self, state: MockState) -> Self {
        self.state = state;
        self
    }

    /// Pre-populates the server with pilots.
    #[must_use]
    pub fn pilots(mut self, pilots: Vec<crate::types::datafeed::Pilot>) -> Self {
        self.state.pilots = pilots;
        self
    }

    /// Pre-populates the server with controllers.
    #[must_use]
    pub fn controllers(mut self, controllers: Vec<crate::types::datafeed::Controller>) -> Self {
        self.state.controllers = controllers;
        self
    }

    /// Pre-populates the server with ATIS stations.
    #[must_use]
    pub fn atis(mut self, atis: Vec<crate::types::datafeed::Atis>) -> Self {
        self.state.atis = atis;
        self
    }

    /// Pre-populates the server with servers.
    #[must_use]
    pub fn servers(mut self, servers: Vec<crate::types::datafeed::Server>) -> Self {
        self.state.servers = servers;
        self
    }

    /// Pre-populates the server with prefiles.
    #[must_use]
    pub fn prefiles(mut self, prefiles: Vec<crate::types::datafeed::Prefile>) -> Self {
        self.state.prefiles = prefiles;
        self
    }

    /// Pre-populates the server with Connect users for OAuth authentication.
    #[must_use]
    pub fn users(mut self, users: Vec<crate::types::connect::ConnectUser>) -> Self {
        self.state.users = users;
        self
    }

    /// Enables CORS and security response headers.
    ///
    /// When enabled, every response includes:
    /// - Permissive CORS headers (any origin, method, and header)
    /// - `X-Content-Type-Options: nosniff`
    /// - `X-Frame-Options: SAMEORIGIN`
    /// - `Referrer-Policy: strict-origin-when-cross-origin`
    ///
    /// Disabled by default; typically only needed for standalone deployment.
    #[must_use]
    pub fn security_headers(mut self, enable: bool) -> Self {
        self.security_headers = enable;
        self
    }

    /// Seeds the server from a [`DataFeed`] JSON dump.
    ///
    /// The feed's entity collections (pilots, controllers, ATIS, etc.)
    /// become the initial state **and** are saved as the seed snapshot,
    /// so calling `POST /api/reset` later restores this data.
    #[must_use]
    pub fn seed(mut self, feed: DataFeed) -> Self {
        self.state = MockState::from(feed);
        self
    }

    /// Builds the [`MockServer`], binding to the configured address.
    ///
    /// # Errors
    ///
    /// Returns an error if the configured address cannot be bound.
    pub async fn build(self) -> std::io::Result<MockServer> {
        let listener = TcpListener::bind(&self.bind_addr).await?;
        let mut state = self.state;
        state.snapshot_seed();
        Ok(MockServer {
            state: Arc::new(RwLock::new(state)),
            listener,
            security_headers: self.security_headers,
        })
    }

    /// Convenience: builds and immediately spawns the server in the background.
    ///
    /// Equivalent to `builder.build().await?.spawn()`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configured address cannot be bound or the
    /// listener's local address cannot be determined.
    pub async fn spawn(self) -> std::io::Result<MockServerHandle> {
        self.build().await?.spawn()
    }
}

impl MockServerHandle {
    /// Returns the base URL of the running mock server (e.g. `http://127.0.0.1:12345`).
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns a shared reference to the mock state, allowing direct reads
    /// and writes from test code without going through HTTP.
    #[must_use]
    pub fn state(&self) -> &SharedState {
        &self.state
    }

    /// Returns a [`VatsimClient`] pre-configured to point at this mock server.
    #[cfg(feature = "client")]
    #[must_use]
    pub fn client(&self) -> VatsimClient {
        VatsimClient::with_mock_base_url(&self.base_url)
    }

    /// Triggers a graceful shutdown of the mock server and waits for it to
    /// finish.
    pub async fn shutdown(mut self) {
        self.shutdown_tx.send(true).ok();
        if let Some(handle) = self.join_handle.take() {
            handle.await.ok();
        }
    }
}

impl Drop for MockServerHandle {
    fn drop(&mut self) {
        // Signal shutdown. The background task will stop on its own;
        // we cannot `.await` here, but the signal is enough.
        self.shutdown_tx.send(true).ok();
    }
}
