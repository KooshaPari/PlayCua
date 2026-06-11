# PlayCua — NVMSCUA Framework Specification

> **Re-frame (2026-06-08):** PlayCua is no longer a "bare-metal computer-use agent" — that was a
> blind narrowing. The framework is **NVMSCUA**: a full computer-use automation (CUA) framework
> with a pluggable **modality** layer (native | sandbox | nvms | wsl | container), a
> WINE-bridge for cross-OS interop, and three top-level surfaces:
>
> 1. **MCP server** — `playcua-mcp` exposes all CUA primitives to Claude Desktop,
>    Cursor, mcp-agent, and any MCP-compatible client.
> 2. **Scriptable CLI** — `playcua-cli` provides a shell-friendly interface for
>    pipelines, CI, and `xargs`/`parallel` workflows.
> 3. **Skill SDK** — Rust trait-based extensibility (`plugins/`), with the same trait
>    surface accessible to Python (`bindings/python`) and C# (`bindings/csharp`).
>
> Bare-metal is **one modality option**, not the goal.

## Goals

| # | Goal | Why |
|---|------|-----|
| G1 | Modality-pluggable execution | Same CUA primitives (screenshot, input, window mgmt) work against native, sandboxed, NVMS-isolated, WSL, or container targets with no caller change. |
| G2 | WINE-bridge interop | Linux host driving Windows binaries, and vice-versa, with WINE's DXvk-style translation. |
| G3 | MCP + CLI + Skill SDK | Every surface area of `trycua/cua` (Python, CLI, library) is matched and improved on. |
| G4 | SOTA quality, not SOTA LOC | Lean implementations, vendored only where forks add real value (e.g. WSL, gVisor). |
| G5 | OpenRPC contract | Every IPC method is in `contracts/openrpc.json`; generated client libraries (Rust/Python/C#) cannot drift. |

## Architecture (target)

```
┌────────────────────────────────────────────────────────────────────┐
│  Surfaces                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                │
│  │ playcua-mcp │  │ playcua-cli │  │  Skill SDK   │                │
│  │   (rmcp)     │  │   (clap)     │  │ (trait + py/ │                │
│  │              │  │              │  │   csharp)    │                │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                │
│         └─────────────────┼─────────────────┘                        │
│                           ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Dispatcher (native/src/ipc/dispatcher.rs)                    │  │
│  │  14 JSON-RPC 2.0 methods, OpenRPC-validated, plugin-augmented│  │
│  └──────────────────────────┬───────────────────────────────────┘  │
│                             ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Modality layer (native/src/modality.rs)                     │  │
│  │  trait Modality { capture, input, windows, process, ... }   │  │
│  │  impls: Native | Sandbox | Nvms | Wsl | Container          │  │
│  └──────────┬────────────┬─────────────┬──────────┬─────────────┘  │
│             ▼            ▼             ▼          ▼                │
│         ┌──────┐    ┌────────┐    ┌────────┐  ┌──────────┐          │
│         │Native│    │Windows │    │  NVMS  │  │Container │          │
│         │ X11  │    │Sandbox │    │(nanovms│  │  (docker │          │
│         │ WGC  │    │ (Win)  │    │ Pheno- │  │   + WINE-│          │
│         │ CG   │    │ (mac)  │    │Compose)│  │  bridge) │          │
│         └──────┘    └────────┘    └────────┘  └──────────┘          │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Wine-bridge (native/src/wine_bridge.rs)                      │  │
│  │  C ABI passthrough: Linux host -> Windows .exe with WINE      │  │
│  │  + DXvk translation: DirectX -> Vulkan (when WINE)            │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

## Components

| Component | Path | Responsibility | Public API |
|-----------|------|----------------|-----------|
| **MCP server** | `native/src/bin/playcua-mcp.rs` | stdio + streamable HTTP MCP transport, registers 14 tools | 14 `#[tool]` methods |
| **CLI** | `native/src/bin/playcua-cli.rs` | shell scriptable subcommand wrapper around JSON-RPC | `playcua-cli {screenshot,click,type,run,ps,windows,diff,hash}` |
| **Daemon** | `native/src/bin/playcua-native.rs` | stdio JSON-RPC **and** Unix-socket daemon mode (configurable) | JSON-RPC 2.0 |
| **Dispatcher** | `native/src/ipc/dispatcher.rs` | central IPC router, plugin-augmented, OpenRPC-validated | `Dispatcher::dispatch(req) -> Response` |
| **Modality** | `native/src/modality.rs` | trait + impls for native/sandbox/nvms/wsl/container | `Modality::capture(&self) -> RgbaImage` |
| **Wine-bridge** | `native/src/wine_bridge.rs` | C ABI passthrough + DXvk translation | `WineBridge::launch_exe(path, args)` |
| **Adapters** | `native/src/adapters/{linux,macos,windows}/` | platform-level capture/input (WGC, X11, CG, etc.) | internal |
| **Plugin SDK** | `native/src/plugins/mod.rs` | trait-based extensibility (register, find, list, replace) | `Plugin` trait |
| **Bindings** | `bindings/python`, `bindings/csharp` | cross-language clients (PyO3 + .NET) | `playcua.execute(action)` |
| **Contracts** | `contracts/openrpc.json` | OpenRPC 1.2 spec, machine-readable | TLA+ invariants |

## Modality registry (proposed)

| Modality | When | Implementation status | Latency profile |
|----------|------|-----------------------|-----------------|
| `native` | direct host execution, lowest overhead | ✅ shipped (Linux/macOS/Windows adapters) | ~5ms / input event |
| `sandbox` | OS-level isolation required (Windows Sandbox, macOS sandbox-exec, Linux bubblewrap) | 🟡 Windows Sandbox config present (`sandbox/`), macOS/Linux sandboxes are TODO | ~25ms / event (sandboxed) |
| `nvms` | cross-OS or hermetic execution, NVMS picks the right Cutdown (native | wsl | container | vm) | 🟡 stub only, depends on PhenoCompose/nanovms (separate agent) | ~50ms / event |
| `wsl` | Windows host executing Linux tools, or vice-versa (WINE-bridged) | ❌ TODO — `wsl = "0.1.0"` crate for detection, `wine = "0.1.0"` (ethanuppal/wine-rs) for C ABI passthrough | ~80ms / event |
| `container` | full filesystem+process isolation, Docker/podman, with WINE-bridge for Windows .exe images | ❌ TODO | ~150ms / event |

Selection: heuristic-driven default, explicit `--modality` CLI/MCP flag, `BARE_MODALITY` env var.

## WINE-bridge (research summary)

**Goal:** Linux host running playcua should be able to drive a Windows binary
(`notepad.exe`) without a Windows VM.

**SOTA options identified (2026-06-08):**

| Crate / project | License | Capability | Maturity |
|-----------------|---------|-----------|----------|
| `wine` (ethanuppal/wine-rs) v0.1.0 | MPL-2.0 | Rust types and FFI to WINE's wineserver | Early — not production-ready but format is right |
| `wine-apc` (v0.0.0) | — | Wineserver protocol from non-Wine Linux processes | Prototype |
| `is_wine` v0.1.2 | MIT | Detect if running under WINE | Stable |
| `wsl` v0.1.0 | — | Detect WSL runtime | Stable |
| `rdp-rs` v0.1.0 / `rdp-rs-2` v0.1.2 | — | Pure Rust RDP client/server | Early but promising |
| WINE (winehq.org) | LGPL | Full Windows API reimplementation | Mature, 25+ years |
| Proton (Valve) | BSD | WINE + DXvk + Steam-specific glue | Mature for gaming, harder for general use |
| DXvk (doitsujin/dxvk) | zlib | Vulkan translation layer for Direct3D 9/10/11 | Mature, drop-in for WINE/Proton |

**Decision:** Adopt **WINE + DXvk** as the implementation for the `wsl` and
`container` modalities. Don't fork WINE — spawn it as a child process and use
the C ABI passthrough. For the Rust side, use the `wine` crate's type-level
APIs and bind to `wineserver` via the `wine-apc` protocol. Track
`ethanuppal/wine-rs` upstream; if it matures, swap our wrapper for direct use.

## Performance targets

| Metric | Target | Note |
|--------|--------|------|
| Single input event (native) | < 5ms | enigo path, no syscall bounce |
| Single input event (sandbox) | < 25ms | 5ms IPC overhead + 20ms sandbox gate |
| Single input event (nvms) | < 50ms | depends on PhenoCompose |
| Single input event (wsl) | < 80ms | WINE startup amortized per session |
| Cold start (native) | < 2s | JSON-RPC stdio fork |
| Cold start (daemon) | < 200ms | Unix-socket, no fork per call |
| MCP tool latency (stdio) | < 5ms | direct in-process dispatch |
| MCP tool latency (HTTP) | < 15ms | single HTTP round-trip |
| Memory (native daemon) | < 50MB | baseline |
| Memory (per modality) | +20-150MB | depends on modality impl |

## Open questions / TODOs

- [ ] Pick the dispatcher's default modality (env var → CLI flag → config file priority)
- [ ] Define Modality trait + register a "nvms" stub that points at nanovms binary
- [ ] Document WineBridge: when to use it, when to use WSL directly, when to use container
- [ ] Plugin SDK doc-comment: how to write a third-party plugin
- [ ] Restore the 500 LOC of dropped test coverage from nanovms in PhenoCompose (separate agent)

## Decision log

See `docs/adr/` for individual decisions (ADR-001 … ADR-N).
