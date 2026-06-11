# ADR-008: MCP server + CLI as first-class surfaces

- Status: Accepted (MCP + CLI shipped in commit 258843a, 3251dc8)
- Date: 2026-06-08
- Related: ADR-006 (modality), SPEC.md (NVMSCUA reframe)

## Context

The original PlayCua sketch exposed CUA primitives via JSON-RPC 2.0 over
stdio. That's the right protocol but the wrong surface. Two surfaces are
concretely more useful:

1. **MCP (Model Context Protocol)** — the de-facto standard for AI agent ↔
   tool communication in 2026. Claude Desktop, Cursor, mcp-agent, and any
   MCP-compatible client (including our own mcp-agent stack in the
   phenotype-org) can drive PlayCua without writing a JSON-RPC pipe of
   their own.
2. **CLI** — shell scriptable subcommand wrapper for pipelines, CI,
   `xargs`/`parallel` workflows, and developers who don't want to spawn
   a subprocess per call.

## Decision

Add two new binaries to the `playcua` workspace:

1. **`playcua-mcp`** — Model Context Protocol server. Wraps the existing
   `Dispatcher` and registers one `#[tool]` per IPC method. Supports both
   stdio (default, for Claude/Cursor subprocess model) and streamable HTTP
   transports.
2. **`playcua-cli`** — scriptable shell client. Spawns `playcua-native`
   as a subprocess and talks newline-delimited JSON-RPC 2.0 over its stdio.

Both are kept **strictly additive** — the existing `playcua-native` stdio
JSON-RPC daemon is unchanged, so all existing consumers keep working.

### Why rmcp 1.7 (and not hand-rolled MCP)

- **Official** — `modelcontextprotocol/rust-sdk` is the canonical Rust
  implementation. 1.7+ is stable and matches the 2025-06-18 MCP spec.
- **Zero hand-rolling of the wire protocol** — `#[tool]` and `#[tool_router]`
  proc-macros derive JSON Schema from field-level doc comments, register
  tools, and handle the `tools/list` / `tools/call` envelope.
- **Two transports in one crate** — stdio and streamable HTTP, no
  hand-rolled server.
- **Tarpc was considered and rejected** — tarpc is a generic RPC framework,
  not MCP. Using it would still require building the MCP envelope by hand.
- **mcp-attr (declarative)** was considered and rejected — too early
  (v0.0.7), no production users.

### Why clap 4 (and not argh, structopt, lexopt)

- **Subcommands** are the right shape (14 IPC methods → 14 subcommands).
  `argh` doesn't have great subcommand ergonomics.
- **Mature** — clap 4 has been stable for years, used by ripgrep, cargo,
  rustup, etc.
- **No-std option** not needed — this is a binary, not a library.

## Consequences

### Positive

- **AI agent interop for free** — Claude Desktop, Cursor, mcp-agent, etc.
  can drive PlayCua via MCP with zero glue code.
- **Scriptability** — `playcua-cli` works inside shell pipelines, xargs,
  parallel, GNU make, Nix builds, GitHub Actions.
- **No regression** — `playcua-native` is unchanged; existing consumers
  keep working.
- **Single source of truth** — the 14 IPC methods are defined once in
  `Dispatcher`, exposed as JSON-RPC methods, MCP tools, and CLI
  subcommands. Adding a new method propagates to all three surfaces
  automatically (MCP and CLI registrations are explicit; we could
  codegen later).

### Negative

- **2 new binaries to maintain** — `playcua-mcp` and `playcua-cli`.
  Each is small (~100-300 LOC), so the maintenance cost is low.
- **MCP/rmcp API churn** — rmcp 1.x is stable but not 1.0. Future major
  versions may require touch-ups. Mitigated by keeping the rmcp
  surface in a single `mcp_server` module behind a feature flag.
- **CLI subprocess per call** is wasteful for tight loops. Mitigated by
  the `playcua-native` Unix-socket daemon mode (added in commit TBD;
  see ADR-009).

## Alternatives considered

- **Hand-rolled JSON-RPC over a long-lived socket** — would have given us
  better latency for the CLI's tight loops, but doesn't help MCP, and
  MCP is the higher-leverage surface.
- **Build the CLI as a library + thin wrapper** — would let other Rust
  tools reuse the client. Skipped: the CLI is intentionally small, and
  the `mcp_server` and `mcp-agent` consumers can use the library form of
  `Dispatcher` directly.
- **No MCP, just expose the JSON-RPC daemon and let the user bridge** —
  rejected because MCP is the de-facto standard and the cost of supporting
  it is small (~100 LOC with rmcp).

## Implementation status

- ✅ `playcua-mcp` (commit 258843a): 14 `#[tool]`-registered methods,
  stdio + streamable HTTP transport, 2 unit tests.
- ✅ `playcua-cli` (commit 3251dc8): 14 subcommands with clap 4, spawns
  `playcua-native` as subprocess, newline-delimited JSON-RPC.
- ⏳ Unix-socket daemon mode for `playcua-native` (next, ADR-009) —
  lets the CLI avoid fork-per-call.
