# ADR-006: Modality abstraction — pluggable execution environment

- Status: Accepted
- Date: 2026-06-08
- Deciders: Koosha, Forge
- Supersedes: implicit assumption that "playcua = bare metal" (from commit ab9d42a)

## Context

`trycua/cua` and the original PlayCua sketch both treat the host machine as the
sole execution environment. That assumption is wrong for the actual users of
this framework:

1. **CI/sandboxed execution** — running CUA tests against a fresh Windows
   Sandbox, a macOS `sandbox-exec` profile, or a Linux `bubblewrap` namespace
   requires the same primitives but with an isolation boundary.
2. **Cross-OS targets** — automating a Linux app from a macOS host, or
   running a Windows-only app from Linux, requires WINE or WSL.
3. **NVMS** — the PhenoCompose/nanovms project is the canonical "pick the
   right Cutdown" runtime. Forcing PlayCua to live on the host machine means
   we can't take advantage of NVMS' automatic native | wsl | container | vm
   selection.
4. **Repeatability** — modal operations (e.g. a test that needs a clean
   Windows install every run) require container/sandbox modalities.

Conflating "computer-use" with "bare metal" is the original sin this ADR
corrects.

## Decision

Introduce a **`Modality` trait** in `native/src/modality.rs` with the same
method surface as the existing IPC methods (`screenshot`, `input.click`,
`windows.list`, `process.launch`, etc.). The dispatcher dispatches to the
currently-selected `Modality` impl instead of calling adapters directly.

Initial impls (in priority order):

| Modality | Backend | Selection criteria | Status |
|----------|---------|--------------------|--------|
| `Native` | existing adapters/ | default, host = target | ✅ shipped |
| `Sandbox` | OS-level sandbox (Windows Sandbox, sandbox-exec, bwrap) | `--modality sandbox` | 🟡 Windows config present |
| `Nvms` | nanovms via PhenoCompose | `--modality nvms` or `BARE_MODALITY=nvms` | 🟡 stub |
| `Wsl` | WSL2 + WINE-bridge | `--modality wsl` | ❌ TODO |
| `Container` | Docker/podman with WINE-bridge for Windows images | `--modality container` | ❌ TODO |

### Trait shape (proposed)

```rust
#[async_trait]
pub trait Modality: Send + Sync {
    fn name(&self) -> &'static str;

    async fn capture(&self) -> Result<RgbaImage, Error>;
    async fn input_click(&self, x: i32, y: i32, button: Button) -> Result<(), Error>;
    async fn input_key(&self, key: Key) -> Result<(), Error>;
    async fn input_type(&self, text: &str) -> Result<(), Error>;
    async fn input_scroll(&self, dx: i32, dy: i32) -> Result<(), Error>;
    async fn windows_list(&self) -> Result<Vec<WindowInfo>, Error>;
    async fn windows_focus(&self, hwnd: u64) -> Result<(), Error>;
    async fn windows_find(&self, title: &str) -> Result<Option<WindowInfo>, Error>;
    async fn process_launch(&self, path: &str, args: &[&str]) -> Result<u32, Error>;
    async fn process_kill(&self, pid: u32) -> Result<(), Error>;
    async fn process_status(&self, pid: u32) -> Result<ProcessInfo, Error>;
    async fn analysis_diff(&self, a: &str, b: &str) -> Result<DiffResult, Error>;
    async fn analysis_hash(&self, path: &str) -> Result<String, Error>;
}

pub struct ModalityRegistry {
    modalities: HashMap<&'static str, Arc<dyn Modality>>,
    default: &'static str,
}

impl ModalityRegistry {
    pub fn dispatch(&self, method: &str, params: Value) -> Result<Value, Error> { ... }
}
```

The dispatcher delegates to `registry.dispatch(method, params)` which becomes
the single integration point with modalities.

### Selection precedence

1. `--modality <name>` CLI/MCP flag
2. `BARE_MODALITY` env var
3. Config file (`~/.config/playcua/modality.toml`)
4. Heuristic (host == target → `native`; explicit Windows binary in WINE → `wine`)

## Consequences

### Positive

- **No caller change** for existing native users — default modality is `native`.
- **Pluggable** — third parties can implement their own `Modality` (e.g. a
  remote-desktop modality, a headless-render modality, a cloud-VM modality)
  and register it at runtime via the plugin SDK.
- **NVMSCUA becomes a single trait impl**, not a rewrite. Adding the
  `Nvms` modality is a ~150-LOC file that calls into nanovms.
- **Tests can be hermetic** — a test that needs a clean Windows install
  runs against `modality = "sandbox"`, no host pollution.

### Negative

- The current 14 IPC methods are concrete (call adapters directly); the
  refactor to call through `ModalityRegistry` is a touch-everything change.
- The trait is async, but the current Dispatcher is sync (one-shot JSON-RPC
  over stdio). The transition will move the dispatcher to tokio or async-std.
- `modality = "wine"` adds a WINE dependency for users who never want it.
  Mitigated by lazy-loading the `wine-bridge` module only when selected.

### Neutral

- Existing 14 IPC methods keep their exact wire signatures and OpenRPC
  contract — only the dispatch path changes.
- The `mcp_server` and `cli` binaries keep their full surface.

## Alternatives considered

- **Per-modality binary** (e.g. `playcua-native-wsl`, `playcua-native-docker`).
  Rejected: proliferates binaries, complicates MCP server config, harder to
  compose.
- **Runtime detection of "the right modality"** (e.g. auto-pick WSL when
  target is a `.exe` on Linux host). Rejected: surprising behavior; the user
  should be explicit. Mitigated by a `--modality auto` mode.
- **Plugin-only** (no built-in modalities, force every user to write their
  own). Rejected: bad DX, no reference impl, no good defaults.

## Implementation plan

1. **M1 (this PR)**: trait + registry + `Native` impl (refactor, no behavior change).
2. **M2**: `Sandbox` modality for Windows (use `sandbox/` configs already present).
3. **M3**: `Nvms` modality stub that calls into nanovms binary at `nanovms` path.
4. **M4**: `Wsl` modality + WINE-bridge research follow-up (see ADR-007).
5. **M5**: `Container` modality with Docker/podman.
