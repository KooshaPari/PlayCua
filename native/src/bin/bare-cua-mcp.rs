//! `bare-cua-mcp` — Model Context Protocol server for bare-cua.
//!
//! Exposes the 14 native IPC methods (`screenshot`, `input.*`, `windows.*`,
//! `process.*`, `analysis.*`) as MCP tools. The protocol server is selected
//! by `--transport {stdio,http}` (default: stdio for Claude Desktop /
//! Cursor / mcp-agent subprocess usage).
//!
//! Examples:
//!   bare-cua-mcp                                    # stdio transport
//!   bare-cua-mcp --transport http --bind 127.0.0.1 --port 3000
//!
//! Build:
//!   cargo build --bin bare-cua-mcp --features mcp-server --release

#![cfg(feature = "mcp-server")]

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use rmcp::transport::streamable_http_server::tower::StreamableHttpService;
use rmcp::ServiceExt;
use tracing::info;
use tracing_subscriber::EnvFilter;

use bare_cua_native::app::App;
use bare_cua_native::mcp_server::BareCuaMcp;

#[derive(Parser, Debug)]
#[command(name = "bare-cua-mcp", version, about = "bare-cua as an MCP server")]
struct Args {
    /// Transport: "stdio" (default, for Claude/Cursor subprocess) or "http"
    /// (streamable HTTP, for multi-client server deployments).
    #[arg(long, default_value = "stdio", value_parser = ["stdio", "http"])]
    transport: String,

    /// Bind address for `transport=http` (ignored on stdio).
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    /// Bind port for `transport=http` (ignored on stdio).
    #[arg(long, default_value_t = 3000)]
    port: u16,

    /// Path prefix for HTTP MCP endpoint. Default: `/mcp`.
    #[arg(long, default_value = "/mcp")]
    path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let args = Args::parse();
    info!(
        version = env!("CARGO_PKG_VERSION"),
        transport = %args.transport,
        "bare-cua-mcp starting"
    );

    // Wire the app once; the Arc is shared across all transports / clients.
    let app = App::build();
    let dispatcher = Arc::new(app.dispatcher);

    match args.transport.as_str() {
        "stdio" => serve_stdio(dispatcher).await,
        "http" => serve_http(dispatcher, &args).await,
        other => {
            anyhow::bail!("Unknown transport: {other}");
        }
    }
}

async fn serve_stdio(dispatcher: Arc<bare_cua_native::ipc::dispatcher::Dispatcher>) -> Result<()> {
    let server = BareCuaMcp::new(dispatcher)
        .serve(rmcp::transport::stdio())
        .await
        .context("failed to start stdio MCP server")?;
    info!("stdio transport ready; waiting for client");
    server.waiting().await?;
    info!("stdio transport closed; exiting");
    Ok(())
}

async fn serve_http(
    dispatcher: Arc<bare_cua_native::ipc::dispatcher::Dispatcher>,
    args: &Args,
) -> Result<()> {
    let path = args.path.clone();
    let svc = StreamableHttpService::new(
        move || Ok(BareCuaMcp::new(Arc::clone(&dispatcher))),
        rmcp::transport::streamable_http_server::session::local::LocalSessionManager::default()
            .into(),
        rmcp::transport::streamable_http_server::StreamableHttpServerConfig::default()
            .with_stateful_mode(false),
    );
    let router = axum::Router::new().nest_service(&path, svc);
    let addr: SocketAddr = format!("{}:{}", args.bind, args.port).parse()?;
    info!(%addr, path, "streamable HTTP transport ready");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::from_env("BARE_CUA_LOG")
                .add_directive("bare_cua_native=info".parse().unwrap())
                .add_directive("rmcp=warn".parse().unwrap()),
        )
        .json()
        .init();
}
