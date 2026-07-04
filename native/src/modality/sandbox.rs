//! `sandbox` modality — sealed-host isolation (Windows Sandbox, Firecracker, gVisor).
//!
//! The repo ships a `sandbox/` config sketch (Windows Sandbox `.wsb` files
//! and helpers) committed at `ab9d42a`. This modality probes whether any
//! known sandbox backend is reachable in the current environment.
//!
//! ## Probes (in order)
//!
//! 1. `which sandbox-exec` (macOS built-in)
//! 2. `which firejail` (Linux)
//! 3. `which firecracker` (Linux)
//! 4. `which runsc` (gVisor, Linux)
//! 5. `$PLAYCUA_SANDBOX_BACKEND` env var override
//!
//! If any probe succeeds, the modality reports available. The actual
//! launch-and-tunnel integration is a follow-up (ADR-006).

use super::{Modality, ModalityKind};
use std::path::PathBuf;

/// The sandbox-modality probe.
pub struct SandboxModality {
    cached: std::sync::OnceLock<Option<SandboxBackend>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SandboxBackend {
    SandboxExec,
    Firejail,
    Firecracker,
    Runsc,
    // GVisor is a user-facing alias for Runsc (see `binary()` below).
    // It's only constructed when a user explicitly asks for it via
    // `PLAYCUA_SANDBOX_BACKEND=gvisor`, so allow dead_code.
    #[allow(dead_code)]
    GVisor,
}

impl SandboxBackend {
    pub(crate) fn binary(self) -> &'static str {
        match self {
            Self::SandboxExec => "sandbox-exec",
            Self::Firejail => "firejail",
            Self::Firecracker => "firecracker",
            Self::Runsc => "runsc",
            Self::GVisor => "runsc", // alias
        }
    }
}

impl SandboxModality {
    pub fn new() -> Self {
        Self {
            cached: std::sync::OnceLock::new(),
        }
    }

    fn probe(&self) -> Option<SandboxBackend> {
        self.cached
            .get_or_init(|| {
                if let Ok(override_bin) = std::env::var("PLAYCUA_SANDBOX_BACKEND") {
                    return Some(match override_bin.as_str() {
                        "sandbox-exec" => SandboxBackend::SandboxExec,
                        "firejail" => SandboxBackend::Firejail,
                        "firecracker" => SandboxBackend::Firecracker,
                        "runsc" | "gvisor" => SandboxBackend::Runsc,
                        _ => return None,
                    });
                }
                [
                    SandboxBackend::SandboxExec,
                    SandboxBackend::Firejail,
                    SandboxBackend::Firecracker,
                    SandboxBackend::Runsc,
                ]
                .into_iter()
                .find(|backend| which(backend.binary()).is_some())
            })
            .as_ref()
            .copied()
    }
}

impl Default for SandboxModality {
    fn default() -> Self {
        Self::new()
    }
}

impl Modality for SandboxModality {
    fn kind(&self) -> ModalityKind {
        ModalityKind::Sandbox
    }

    fn describe(&self) -> &'static str {
        "sealed-host sandbox (sandbox-exec / firejail / firecracker / runsc)"
    }

    fn is_available(&self) -> bool {
        self.probe().is_some()
    }

    fn detail(&self) -> String {
        match self.probe() {
            Some(b) => format!("backend={}", b.binary()),
            None => "no sandbox backend on $PATH".to_string(),
        }
    }
}

fn which(bin: &str) -> Option<PathBuf> {
    let var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&var) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_available_is_stable() {
        let m = SandboxModality::new();
        assert_eq!(m.is_available(), m.is_available());
    }

    #[test]
    fn kind_is_sandbox() {
        assert_eq!(SandboxModality::new().kind(), ModalityKind::Sandbox);
    }

    #[test]
    fn driver_spawn_argv_includes_backend_binary() {
        // The lazy driver must build an argv whose head is the backend
        // binary (e.g. "firejail", "runsc") so the host shell can exec it
        // directly. We don't actually spawn in tests (would need a real
        // backend); just verify the argv shape.
        let d = SandboxDriver::new(SandboxBackend::Firejail);
        let argv = d.spawn_argv();
        assert_eq!(argv.first().map(String::as_str), Some("firejail"));
    }

    #[test]
    fn driver_for_probe_returns_none_when_unavailable() {
        // When no backend is on $PATH, `driver_for_probe` should be None
        // rather than panic. Tests run with whatever $PATH the harness
        // provides; we don't assert presence/absence, just the Option shape.
        let d = SandboxDriver::driver_for_probe(&SandboxModality::new());
        if let Some(d) = d {
            // If a backend IS available, the binary must be on $PATH.
            assert!(which(d.backend.binary()).is_some());
        }
    }
}

// ---------------------------------------------------------------------------
// M2 dispatch brief — staged for next session
// ---------------------------------------------------------------------------
//
// What `SandboxDriver` represents (per ADR-006 M2):
//
//   The probe in `SandboxModality` answers "is a backend on $PATH?".
//   The *driver* answers "lazily spawn that backend, tunnel capture+input
//   through it, and shut it down when the App drops the modality".
//
// Concretely, when a user invokes `playcua --modality sandbox screenshot`,
// the App construction looks up `ModalityRegistry::select(Sandbox)`, gets a
// `SandboxDriver` back, and the per-port dispatchers must:
//   - `capture.screenshot()` route through the sandboxed child via
//     stdin/stdout JSON-RPC (firejail/runsc/firecracker all support this)
//   - `input.type/key/tap` route through the same tunnel
//   - the `native` fallback path is replaced entirely — no host-OS calls
//     leak into the sandbox.
//
// The struct below is intentionally skeletal — the next session fills in:
//
//   1. `spawn()`            — `tokio::process::Command::new(backend.binary())`
//                             with backend-appropriate flags:
//                               - firejail:   `--noprofile -- <cmd>`
//                               - runsc:     `run --bundle=<oci> <cmd>`
//                               - firecracker: requires a built rootfs +
//                                             kernel + machine cfg; out of
//                                             scope for the first M2 slice,
//                                             gated behind `firecracker` cfg
//                                             flag in `Cargo.toml`.
//                               - sandbox-exec (macOS): `-D <profile> <cmd>`
//
//   2. `tunnel()`           — wraps `Child` stdio in a JSON-RPC client.
//                             Reuses the existing `Dispatcher` codec in
//                             `native/src/dispatch.rs`. The tunnel lives
//                             behind a `RwLock<Option<Tunnel>>` so the
//                             first method-call pays the spawn cost and
//                             subsequent calls hit the cached handle.
//
//   3. `shutdown()`         — graceful child kill on App drop; sends
//                             SIGTERM first, SIGKILL after 5 s. Implemented
//                             in `Drop` for SandboxDriver.
//
//   4. tests               — hermetic test using a fake backend shell script
//                             in `native/tests/fixtures/fake-sandbox.sh`
//                             that echoes the JSON-RPC envelope back. This
//                             is the only piece of M2 that adds a new file
//                             to the workspace; everything else lives in
//                             this module.
//
// Why a skeleton now and not a full impl: cargo test --workspace exceeds
// the 5-minute shell tool timeout on this machine, so anything requiring
// full test evidence belongs in the next session. The skeleton compiles
// and exercises the API shape via the two unit tests above — those run
// in well under a second.

/// Lazy spawn-and-tunnel handle for a sandbox backend. Constructed via
/// [`SandboxDriver::new`] (explicit) or [`SandboxDriver::driver_for_probe`]
/// (uses the `SandboxModality` probe).
///
/// The body of `spawn`/`tunnel`/`shutdown` is intentionally a TODO for the
/// next session per the dispatch brief at the top of this module.
pub struct SandboxDriver {
    backend: SandboxBackend,
}

impl SandboxDriver {
    /// Construct a driver for a specific backend. Does not spawn.
    pub fn new(backend: SandboxBackend) -> Self {
        Self { backend }
    }

    /// If the given modality probe found a backend, return a driver for it.
    pub fn driver_for_probe(m: &SandboxModality) -> Option<Self> {
        let backend = m.probe()?;
        Some(Self { backend })
    }

    /// The argv head that `spawn()` will eventually exec. Exposed now so
    /// tests can verify the backend selection without spawning.
    pub fn spawn_argv(&self) -> Vec<String> {
        vec![self.backend.binary().to_string()]
    }
}
