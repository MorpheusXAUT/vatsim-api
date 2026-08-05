//! Standalone mock VATSIM server.
//!
//! Serves the data feed, slurper, and Connect OAuth endpoints against in-memory
//! state, plus a management API for manipulating that state at runtime. See the
//! `vatsim_api::mock` module documentation for the full endpoint list.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use tracing_subscriber::EnvFilter;
use vatsim_api::mock::MockServer;
use vatsim_api::mock::state::MockState;

/// Mock VATSIM API server for integration testing.
#[derive(Debug, Parser)]
#[command(
    name = "vatsim-mock",
    version,
    about = "Mock VATSIM API server for integration testing",
    long_about = None,
)]
struct Args {
    /// Address to bind to.
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    bind: String,

    /// Path to a JSON seed file to preload the server state from.
    ///
    /// The file uses the `MockState` shape: an object whose keys are the entity
    /// collections (`controllers`, `pilots`, `atis`, `servers`, `prefiles`,
    /// `users`, ...). Every key is optional. The loaded state also becomes the
    /// snapshot that `POST /api/reset` restores.
    #[arg(short, long, value_name = "PATH")]
    seed: Option<PathBuf>,

    /// Log filter, in `RUST_LOG` syntax.
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&args.log))
        .with_writer(std::io::stderr)
        .init();

    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{e}");
            ExitCode::FAILURE
        }
    }
}

type BoxError = Box<dyn std::error::Error>;

async fn run(args: Args) -> Result<(), BoxError> {
    let mut builder = MockServer::builder()
        .bind(&args.bind)
        .security_headers(true);

    if let Some(path) = &args.seed {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read seed file {}: {e}", path.display()))?;
        let state: MockState = serde_json::from_str(&data)
            .map_err(|e| format!("failed to parse seed file {}: {e}", path.display()))?;

        let entities =
            state.pilots.len() + state.controllers.len() + state.atis.len() + state.prefiles.len();
        tracing::info!(
            seed = %path.display(),
            entities,
            users = state.users.len(),
            "loaded seed file"
        );

        builder = builder.state(state);
    }

    let server = builder
        .build()
        .await
        .map_err(|e| format!("failed to bind to {}: {e}", args.bind))?;

    let addr = server.local_addr()?;
    tracing::info!("listening on http://{addr}");

    server.serve_with_shutdown(shutdown_signal()).await?;
    tracing::info!("shut down");

    Ok(())
}

/// Resolves when the process receives Ctrl-C, or SIGTERM on Unix.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
