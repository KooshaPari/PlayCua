//! bare-cua-native: stdio JSON-RPC 2.0 server (or Unix-socket daemon)
//! for computer-use automation.
//!
//! Two modes:
//! - **stdio** (default): reads newline-delimited JSON-RPC 2.0 requests from
//!   stdin, dispatches to platform-selected port adapters via the hexagonal
//!   architecture, writes responses to stdout. All logging goes to stderr
//!   (JSON format). This is the mode `bare-cua-cli` invokes per call.
//! - **daemon** (`--socket <path>`): binds a Unix-domain socket at `path`,
//!   accepts concurrent client connections, and serves the same JSON-RPC
//!   2.0 protocol on each. Stale socket files are removed first; the
//!   socket file is cleaned up on Ctrl-C or fatal error. This is the
//!   mode `bare-cua-cli --daemon` will use for tight loops.
//!
//! Mode selection is by argv (positional, not flag) so the binary stays
//! drop-in compatible with shell pipelines.

use std::sync::Arc;

use anyhow::Result;
use ipc::{read_request, write_response};
use tokio::io::{self, AsyncWriteExt, BufReader};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod adapters;
mod app;
mod domain;
mod ipc;
mod plugins;
mod ports;
#[cfg(unix)]
mod socket;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing to stderr in JSON format.
    // Level is controlled by BARE_CUA_LOG env var (default: info).
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::from_env("BARE_CUA_LOG")
                .add_directive("bare_cua_native=info".parse().unwrap()),
        )
        .json()
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "bare-cua-native starting"
    );

    // Wire up all adapters via DI. Wrapped in Arc so the daemon mode
    // can hand a cheap clone to each connection handler.
    let app = Arc::new(app::App::build());

    // Mode dispatch: argv[1] is "--socket" (with argv[2] = path) for daemon
    // mode, absent for stdio mode. The stdio mode is fully back-compatible
    // with the original CLI invocation pattern.
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--socket" {
        #[cfg(unix)]
        {
            let socket_path = std::path::PathBuf::from(&args[2]);
            return socket::run(app, socket_path).await;
        }
        #[cfg(not(unix))]
        {
            error!("--socket mode is Unix-only (Linux/macOS). Build the daemon differently for Windows.");
            std::process::exit(2);
        }
    }

    run_stdio(app).await
}

/// Stdio JSON-RPC 2.0 loop (the original `bare-cua-native` mode).
async fn run_stdio(app: Arc<app::App>) -> Result<()> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout);

    loop {
        let req = match read_request(&mut reader).await {
            Ok(Some(r)) => r,
            Ok(None) => {
                info!("stdin EOF — shutting down");
                break;
            }
            Err(e) => {
                error!(error = %e, "Failed to parse request");
                let resp = ipc::Response::err(
                    serde_json::Value::Null,
                    -32700,
                    format!("Parse error: {e}"),
                );
                let _ = write_response(&mut writer, &resp).await;
                continue;
            }
        };

        let resp = app.dispatcher.dispatch(req).await;

        if let Err(e) = write_response(&mut writer, &resp).await {
            error!(error = %e, "Failed to write response");
            break;
        }
    }

    writer.flush().await?;
    info!("bare-cua-native exiting");
    Ok(())
}
