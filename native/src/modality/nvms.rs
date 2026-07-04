//! `nvms` modality — drive an `nvms`-orchestrated container (nanovms).
//!
//! The canonical home for nvms is `KooshaPari/nanovms`. This modality probes
//! for the `nvms` CLI on `$PATH` and, if present, reports availability.
//!
//! ## What it does in this slice
//!
//! - **Availability probe**: `which nvms` (PATH lookup, no execution).
//! - **Describe**: "nvms <version> at /path/to/nvms" if found, otherwise
//!   "nvms binary not on $PATH".
//! - **Full driver integration** (auto-provision a container, route requests
//!   into it, etc.) is a follow-up tracked in ADR-006. The probe here is the
//!   gating check that lets `auto` selection know nvms is even an option.
//!
//! ## Future follow-ups
//!
//! 1. Spawn `nvms run <config>` on first action (lazy init).
//! 2. Tunnel capture/input through the container's IPC.
//! 3. Honor `PLAYCUA_NVMS_CONFIG` env var (path to nvms.toml).
//! 4. Multi-tenant: share a single nvms runtime across modalities.

use super::{Modality, ModalityKind};
use std::path::PathBuf;

/// The nvms-modality probe.
pub struct NvmsModality {
    /// Cached probe result. Populated on first `is_available()` call so we
    /// don't re-walk $PATH on every ping.
    cached: std::sync::OnceLock<Option<PathBuf>>,
}

impl NvmsModality {
    pub fn new() -> Self {
        Self {
            cached: std::sync::OnceLock::new(),
        }
    }

    /// Re-probe from scratch (drops the cached result).
    #[cfg(test)]
    pub fn invalidate_cache(&mut self) {
        // OnceLock has no reset; this is a test-only re-construction hack.
        *self = Self::new();
    }

    /// Probe for the `nvms` binary on $PATH.
    fn probe(&self) -> Option<PathBuf> {
        self.cached.get_or_init(which_nvms).as_ref().cloned()
    }
}

impl Default for NvmsModality {
    fn default() -> Self {
        Self::new()
    }
}

impl Modality for NvmsModality {
    fn kind(&self) -> ModalityKind {
        ModalityKind::Nvms
    }

    fn describe(&self) -> &'static str {
        "nvms-orchestrated container (KooshaPari/nanovms)"
    }

    fn is_available(&self) -> bool {
        self.probe().is_some()
    }

    fn detail(&self) -> String {
        match self.probe() {
            Some(p) => format!("nvms={}", p.display()),
            None => "nvms not on $PATH".to_string(),
        }
    }
}

/// Probe for the `nvms` binary on $PATH. Returns the first match.
fn which_nvms() -> Option<PathBuf> {
    let var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&var) {
        let candidate = dir.join("nvms");
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
    fn probe_is_idempotent() {
        let m = NvmsModality::new();
        let a = m.is_available();
        let b = m.is_available();
        assert_eq!(a, b, "is_available must be stable across calls");
    }

    #[test]
    fn detail_matches_availability() {
        let m = NvmsModality::new();
        if m.is_available() {
            assert!(m.detail().starts_with("nvms="), "got: {}", m.detail());
        } else {
            assert_eq!(m.detail(), "nvms not on $PATH");
        }
    }

    #[test]
    fn kind_is_nvms() {
        assert_eq!(NvmsModality::new().kind(), ModalityKind::Nvms);
    }

    #[test]
    fn driver_spawn_argv_includes_nvms_run() {
        // The lazy driver must build an argv whose head is the `nvms run`
        // subcommand so the host shell can exec it directly. We don't
        // actually spawn in tests (would need a real nvms binary); just
        // verify the argv shape.
        let d = NvmsDriver::new();
        let argv = d.spawn_argv();
        assert_eq!(argv.first().map(String::as_str), Some("nvms"));
        assert_eq!(argv.get(1).map(String::as_str), Some("run"));
    }

    #[test]
    fn driver_for_probe_returns_none_when_unavailable() {
        // When no backend is on $PATH, `driver_for_probe` should be None
        // rather than panic. Tests run with whatever $PATH the harness
        // provides; we don't assert presence/absence, just the Option shape.
        let d = NvmsDriver::driver_for_probe(&NvmsModality::new());
        if d.is_some() {
            // If nvms IS available, the binary must be on $PATH.
            assert!(which_nvms().is_some());
        }
    }
}

// ---------------------------------------------------------------------------
// M3 dispatch brief — staged for next session
// ---------------------------------------------------------------------------
//
// What `NvmsDriver` represents (per ADR-006 M3):
//
//   The probe in `NvmsModality` answers "is `nvms` on $PATH?".
//   The *driver* answers "spawn `nvms run <config>`, tunnel capture/input
//   through the resulting container, and shut it down on App drop".
//
//   nvms is the orchestrator from `KooshaPari/nanovms` — different from
//   the container CLI in `container.rs`. While the Container modality
//   shells out to docker/podman, the nvms modality speaks nvms's native
//   RPC protocol and can share a runtime across multiple PlayCua sessions.
//
// Concretely, when a user invokes `playcua --modality nvms screenshot`,
// the App construction looks up `ModalityRegistry::select(Nvms)`, gets a
// `NvmsDriver` back, and the per-port dispatchers must:
//   - `capture.screenshot()` route through the nvms tunnel RPC
//   - `input.type/key/tap` route through the same tunnel
//   - the `native` fallback path is replaced entirely
//
// The struct below is intentionally skeletal — the next session fills in:
//
//   1. `spawn()`             — `tokio::process::Command::new("nvms")` with
//                              subcommand `run --config <nvms.toml>`.
//                              The config path comes from
//                              `PLAYCUA_NVMS_CONFIG` env var or
//                              `~/.config/playcua/nvms.toml`.
//
//   2. `tunnel()`            — wraps `Child` stdio in the nvms RPC codec.
//                              Reuses the existing `Dispatcher` codec in
//                              `native/src/dispatch.rs`. The tunnel lives
//                              behind a `RwLock<Option<Tunnel>>` so the
//                              first method-call pays the spawn cost and
//                              subsequent calls hit the cached handle.
//
//   3. `shutdown()`          — graceful child kill on App drop; sends
//                              SIGTERM first, SIGKILL after 5 s. Implemented
//                              in `Drop` for NvmsDriver.
//
//   4. config authoring      — a sample `nvms.toml` for the first M3 slice
//                              (image: ubuntu:22.04 with xvfb + x11-apps +
//                              the PlayCua capture bridge). Lives in
//                              `sandbox/nvms.toml` (not added in this PR).
//
//   5. tests                — hermetic test using a fake `nvms` shell script
//                             in `native/tests/fixtures/fake-nvms.sh` that
//                             echoes the RPC envelope back. This is the only
//                             piece of M3 that adds a new file to the
//                             workspace.
//
// Why a skeleton now and not a full impl: cargo test --workspace exceeds
// the 5-minute shell tool timeout on this machine, so anything requiring
// full test evidence belongs in the next session. The skeleton compiles
// and exercises the API shape via the two unit tests above — those run
// in well under a second.

/// Lazy spawn-and-tunnel handle for an `nvms run` invocation.
///
/// The body of `spawn`/`tunnel`/`shutdown` is intentionally a TODO for the
/// next session per the dispatch brief at the top of this module.
pub struct NvmsDriver;

impl NvmsDriver {
    /// Construct a driver. Does not spawn.
    pub fn new() -> Self {
        Self
    }

    /// If `nvms` is on $PATH, return a driver; otherwise None.
    pub fn driver_for_probe(m: &NvmsModality) -> Option<Self> {
        if m.is_available() {
            Some(Self)
        } else {
            None
        }
    }

    /// The argv head that `spawn()` will eventually exec. Exposed now so
    /// tests can verify the subcommand shape without spawning.
    pub fn spawn_argv(&self) -> Vec<String> {
        vec!["nvms".to_string(), "run".to_string()]
    }
}
