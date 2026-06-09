//! bare-cua-native: stdio JSON-RPC 2.0 server (or Unix-socket daemon)
//! for computer-use automation.
//!
//! Three mode flags (all optional, all mutually composable):
//! - **stdio** (default): reads newline-delimited JSON-RPC 2.0 requests from
//!   stdin, dispatches to platform-selected port adapters via the hexagonal
//!   architecture, writes responses to stdout. All logging goes to stderr
//!   (JSON format). This is the mode `bare-cua-cli` invokes per call.
//! - **daemon** (`--socket <path>`): binds a Unix-domain socket at `path`,
//!   accepts concurrent client connections, and serves the same JSON-RPC
//!   2.0 protocol on each. Stale socket files are removed first; the
//!   socket file is cleaned up on Ctrl-C or fatal error. This is the
//!   mode `bare-cua-cli --daemon` will use for tight loops.
//! - **modality** (`--modality <kind>`): selects the runtime environment
//!   (native | sandbox | nvms | wsl | container) per the NVMSCUA framework.
//!   See `modality/` and ADR-006 for the full design. Falls back to `auto`
//!   (env var or host-OS heuristic) when unset.
//!
//! Mode selection is by argv (positional, not flag) so the binary stays
//! drop-in compatible with shell pipelines.

use std::sync::Arc;

use anyhow::Result;
use bare_cua_native::app;
use bare_cua_native::ipc;
use bare_cua_native::ipc::{read_request, write_response};
use bare_cua_native::modality::ModalityKind;
use bare_cua_native::modality::{ModalityEnv, ModalityRegistry};
use tokio::io::{self, AsyncWriteExt, BufReader};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod adapters;
mod domain;
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

    // Parse --modality flag from argv (before the --socket switch).
    // Args: [0] = binary, [1] = "--modality" (optional), [2] = KIND (if [1]),
    //       then optionally [3] = "--socket", [4] = PATH.
    let args: Vec<String> = std::env::args().collect();
    let (modality_flag, rest_args) = parse_modality_arg(&args);

    // Build the modality registry and run selection.
    let mut registry = ModalityRegistry::with_defaults();
    let env = ModalityEnv::from_process_env(modality_flag);
    let selected = registry.select(&env).clone();
    info!(
        kind = %selected.kind,
        describe = %selected.describe,
        detail = %selected.detail,
        available = selected.available,
        "modality selected"
    );

    // Wire up all adapters via DI. Wrapped in Arc so the daemon mode
    // can hand a cheap clone to each connection handler.
    let app = Arc::new(app::App::build(selected));

    // Mode dispatch: --socket <path> for daemon mode, absent for stdio.
    if rest_args.len() >= 2 && rest_args[0] == "--socket" {
        #[cfg(unix)]
        {
            let socket_path = std::path::PathBuf::from(&rest_args[1]);
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

/// Pull `--modality <KIND>` off the front of `args`. Returns the parsed
/// kind (or None) plus the remaining args (always with `args[0]` stripped).
///
/// Recognized forms:
/// - `bare-cua-native --modality nvms`               -> kind=Nvms, rest=[]
/// - `bare-cua-native --modality nvms --socket p`    -> kind=Nvms, rest=[--socket, p]
/// - `bare-cua-native`                               -> kind=None, rest=[]
/// - `bare-cua-native --socket p`                    -> kind=None, rest=[--socket, p]
fn parse_modality_arg(args: &[String]) -> (Option<ModalityKind>, &[String]) {
    let rest = args.get(1..).unwrap_or(&[]);
    if rest.len() >= 2 && rest[0] == "--modality" {
        match ModalityKind::parse(&rest[1]) {
            Some(k) => (Some(k), &rest[2..]),
            None => {
                // Unknown kind: fall through to env/auto. Don't error — keeps
                // the binary drop-in for typos.
                (None, &rest[2..])
            }
        }
    } else {
        (None, rest)
    }
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
