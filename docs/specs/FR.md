# PlayCua — Functional Requirements (FR)

> **Phase 3 traceability layer.** Every FR is anchored to a real piece of
> code in this workspace. See `docs/specs/TRACEABILITY.md` for the
> FR → source ↔ test cross-reference.

## Scope

This document enumerates the user-visible functional requirements that
PlayCua's `playcua-native` core must satisfy. The requirements are
grounded in the existing architecture:

- **JSON-RPC 2.0 dispatcher** (`native/src/ipc/dispatcher.rs`) — the
  central router. Exposes 14 methods over stdio and (configurable)
  Unix-socket daemon mode.
- **Modality layer** (`native/src/modality/`) — pluggable execution
  targets: `native | sandbox | nvms | wsl | container`.
- **Hexagonal port traits** (`native/src/ports/`) — `CapturePort`,
  `InputPort`, `WindowPort`, `ProcessPort`, `AnalysisPort`. Adapters
  live in `native/src/adapters/`.
- **Domain types** (`native/src/domain/`) — `Frame`, `DiffResult`,
  `HashResult`, `Key`, `KeyAction`, `MouseEvent`, `ProcessHandle`,
  `WindowInfo`, etc.
- **Surfaces** — `playcua-mcp` (rmcp), `playcua-cli` (clap), and the
  Rust/Python/C# bindings.

The FRs below are written against what the **current code** does, not
what a future vision-document promised. Each one cites a real
implementation anchor.

---

## FR-001 — Screen Capture (Display + Window)

**Statement:** The system shall capture a display or a window as a
base64-encoded PNG frame with width and height metadata, and route
the call to the `CapturePort` trait.

**Anchor:**
- Dispatcher method: `handle_screenshot`
  (`native/src/ipc/dispatcher.rs:104-131`)
- Domain type: `Frame { data, width, height }`
  (`native/src/domain/capture.rs:4-10`)
- Port trait: `CapturePort::capture_display(monitor: u32)` /
  `capture_window(title: Option<&str>)`
  (`native/src/ports/capture.rs`)
- Adapter: `NativeCaptureAdapter` in
  `native/src/adapters/xcap.rs` (Linux: `x11.rs`, Windows: `wgc.rs`)

**Acceptance criteria:**
- `screenshot` JSON-RPC call without params returns the primary
  monitor as a PNG.
- `screenshot` with `{ "monitor": N }` returns display N.
- `screenshot` with `{ "window_title": "..." }` captures the matched
  window.
- Response shape: `{ data: base64(png), width, height, format: "png" }`.

---

## FR-002 — Input Injection (Key, Type, Click, Scroll, Move)

**Statement:** The system shall inject keyboard, text, mouse-click,
scroll, and move events via the `InputPort` trait, exposed as five
JSON-RPC methods.

**Anchor:**
- Dispatcher methods: `input.key`, `input.type`, `input.click`,
  `input.scroll`, `input.move` (`native/src/ipc/dispatcher.rs:76-80`)
- Domain types: `Key`, `KeyAction { Press | Down | Up }`,
  `MouseButton`, `MouseAction`, `MouseEvent`, `ScrollDirection`
  (`native/src/domain/input.rs`)
- Port trait: `InputPort` (`native/src/ports/input.rs`)
- Adapter: `EnigoInputAdapter` (`native/src/adapters/enigo.rs`)

**Acceptance criteria:**
- `input.key` accepts `{ key, action: "press" | "down" | "up" }`.
- `input.type` accepts `{ text }` and emits one key event per char.
- `input.click` accepts `{ x, y, button?, action? }`.
- `input.scroll` accepts `{ dx, dy }`.
- `input.move` accepts `{ x, y }`.
- All five return `{ ok: true }` on success, or
  `Response::internal_error` on adapter failure.

---

## FR-003 — Window Enumeration, Focus, and Lookup

**Statement:** The system shall list, focus, and find windows on the
current desktop, with filtering by title substring, by application
name, or by visibility.

**Anchor:**
- Dispatcher methods: `windows.list`, `windows.focus`, `windows.find`
  (`native/src/ipc/dispatcher.rs:82-84`)
- Domain type: `WindowInfo`, `WindowFilter`, `WindowHandle`
  (`native/src/domain/window.rs`)
- Port trait: `WindowPort` (`native/src/ports/window.rs`)
- Adapters:
  - Linux: `ewmh.rs` (EWMH/NetWM) and `enumwin.rs`
  - Windows: `enumwin.rs`
  - macOS: `nsworkspace.rs`

**Acceptance criteria:**
- `windows.list` returns `[{ handle, title, app, visible, ... }]`.
- `windows.focus` accepts `{ handle | title }` and brings window to
  front.
- `windows.find` accepts `{ title?, app?, visible? }` and returns the
  first matching `WindowInfo`.
- `WindowNotFound` surfaces as a typed error mapped to
  `Response::internal_error` with a descriptive message.

---

## FR-004 — Process Lifecycle (Launch, Kill, Status)

**Statement:** The system shall launch, kill, and query the status of
child processes via the `ProcessPort` trait.

**Anchor:**
- Dispatcher methods: `process.launch`, `process.kill`,
  `process.status` (`native/src/ipc/dispatcher.rs:86-88`)
- Domain type: `ProcessHandle`, `ProcessStatus`
  (`native/src/domain/process.rs`)
- Port trait: `ProcessPort` (`native/src/ports/process.rs`)
- Adapter: `ProcessAdapter` (`native/src/adapters/process_adapter.rs`)

**Acceptance criteria:**
- `process.launch` accepts `{ command, args?, cwd?, env? }` and
  returns `{ handle, pid }`.
- `process.kill` accepts `{ handle }` and terminates the process.
- `process.status` accepts `{ handle }` and returns
  `{ running, exit_code? }`.
- A process that exits cleanly has `running: false` and a populated
  `exit_code`.

---

## FR-005 — Image Analysis: Diff and Hash

**Statement:** The system shall compute a perceptual diff ratio
between two PNG frames and a content-addressable BLAKE3 hash of a
single frame, via the `AnalysisPort` trait.

**Anchor:**
- Dispatcher methods: `analysis.diff`, `analysis.hash`
  (`native/src/ipc/dispatcher.rs:90-91`)
- Domain types: `DiffResult { changed, change_ratio }`,
  `HashResult { hash: String }` (`native/src/domain/analysis.rs:4-17`)
- Port trait: `AnalysisPort` (`native/src/ports/analysis.rs`)
- Adapter: `NativeAnalysisAdapter`
  (`native/src/adapters/analysis_adapter.rs`)

**Acceptance criteria:**
- `analysis.diff` accepts `{ a: base64(png), b: base64(png),
  threshold? }` and returns `{ changed, change_ratio }` where
  `change_ratio ∈ [0.0, 1.0]`.
- Two identical frames produce `change_ratio == 0.0` and
  `changed == false`.
- `analysis.hash` accepts `{ frame: base64(png) }` and returns
  `{ hash: hex_string }` (BLAKE3, lowercase hex).
- Dimension mismatches surface as
  `AnalysisError::DimensionMismatch(w1, h1, w2, h2)`.

---

## FR-006 — Modality Selection and Self-Reporting

**Statement:** The system shall select a modality (native | sandbox |
nvms | wsl | container) at startup, defaulting to a heuristic
auto-detect, and shall report the active modality via the `ping`
method.

**Anchor:**
- Dispatcher method: `ping`
  (`native/src/ipc/dispatcher.rs:57-72`)
- Modality registry: `SelectedModality`
  (`native/src/modality/registry.rs`)
- Modality parsers: `ModalityKind::parse("native" | "sandbox" | "nvms"
  | "wsl" | "container")` (`native/src/modality/mod.rs`)
- Override knobs: `--modality` CLI/MCP flag, `BARE_MODALITY` env var
  (per `SPEC.md:124`)

**Acceptance criteria:**
- `ping` returns `{ ok: true, version, modality: { kind, describe,
  detail, available } }` for any selected modality.
- `ModalityKind::parse("native")` returns
  `Ok(ModalityKind::Native)`.
- Unknown modality strings produce a typed error and a non-zero
  exit.

---

## FR-007 — Hexagonal Boundary (Architecture Enforcement)

**Statement:** Application-layer code (the dispatcher) shall depend
only on the port traits declared in `playcua_native::ports`, never on
concrete adapter implementations in `playcua_native::adapters`.

**Anchor:**
- Dispatcher field types: `Arc<dyn CapturePort>`, `Arc<dyn
  InputPort>`, `Arc<dyn WindowPort>`, `Arc<dyn ProcessPort>`,
  `Arc<dyn AnalysisPort>`
  (`native/src/ipc/dispatcher.rs:19-29`)
- Test: `native/tests/hexagonal_boundary.rs` — builds five in-test
  mock port impls and drives the dispatcher with them.

**Acceptance criteria:**
- The dispatcher file imports nothing from `crate::adapters::*`.
- The boundary test compiles and all five mock-driven methods
  (capture, input, windows, process, analysis) return the values the
  mocks emit.
- A future contributor who adds `use crate::adapters::xcap::*;` to
  the dispatcher module triggers a CI lint failure on a paired
  forbid rule.

---

## Cross-references

- Architecture diagram: `SPEC.md:61-97`
- Component table: `SPEC.md:99-112`
- Modality registry: `SPEC.md:114-124`
- Performance targets: `SPEC.md:150-163`
- Traceability index: `docs/specs/TRACEABILITY.md`
