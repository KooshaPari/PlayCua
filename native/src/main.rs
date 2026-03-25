//! bare-cua-native: stdio JSON-RPC 2.0 server for computer-use automation.
//!
//! Reads newline-delimited JSON-RPC 2.0 requests from stdin, dispatches to
//! platform-selected port adapters via the hexagonal architecture, and writes
//! responses to stdout. All logging goes to stderr (JSON format).

mod adapters;
mod app;
mod domain;
mod ipc;
mod plugins;
mod ports;

use anyhow::Result;
use ipc::{read_request, write_response};
use tokio::io::{self, AsyncWriteExt, BufReader};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

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

    // Wire up all adapters via DI.
    let app = app::App::build();

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
