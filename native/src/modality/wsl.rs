//! `wsl` modality — Windows Subsystem for Linux (Windows host only).
//!
//! Probes for `wsl.exe` (Windows) or `wsl` (Linux WSL interop binary, rare).
//! On non-Windows hosts the modality always reports unavailable.

use super::{Modality, ModalityKind};

/// The wsl-modality probe.
pub struct WslModality;

impl WslModality {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WslModality {
    fn default() -> Self {
        Self::new()
    }
}

impl Modality for WslModality {
    fn kind(&self) -> ModalityKind {
        ModalityKind::Wsl
    }

    fn describe(&self) -> &'static str {
        "Windows Subsystem for Linux (wsl.exe)"
    }

    fn is_available(&self) -> bool {
        if cfg!(target_os = "windows") {
            // On Windows, wsl.exe is always reachable if WSL is enabled.
            // We could shell out to `wsl --status` to be precise, but the
            // PATH lookup is sufficient as a first-pass probe.
            std::env::var_os("PATH")
                .map(|p| {
                    std::env::split_paths(&p)
                        .any(|d| d.join("wsl.exe").exists() || d.join("wsl").exists())
                })
                .unwrap_or(false)
        } else {
            false
        }
    }

    fn detail(&self) -> String {
        if cfg!(target_os = "windows") {
            if self.is_available() {
                "wsl.exe reachable".to_string()
            } else {
                "wsl.exe not on $PATH".to_string()
            }
        } else {
            "not Windows".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_is_wsl() {
        assert_eq!(WslModality::new().kind(), ModalityKind::Wsl);
    }

    #[test]
    fn on_non_windows_always_unavailable() {
        if !cfg!(target_os = "windows") {
            assert!(!WslModality::new().is_available());
            assert_eq!(WslModality::new().detail(), "not Windows");
        }
    }

    #[test]
    fn driver_spawn_argv_includes_wsl_exe() {
        // The lazy driver must build an argv whose head is `wsl.exe` so the
        // host shell can exec it directly. We don't actually spawn in tests
        // (would need a real wsl.exe); just verify the argv shape.
        //
        // Note: on non-Windows hosts the driver still constructs but the
        // exec will fail at App construction — that's the expected modality
        // behavior (registry::select returns the Wsl entry only on Windows
        // when `is_available` returns true).
        let d = WslDriver::new();
        let argv = d.spawn_argv();
        assert_eq!(argv.first().map(String::as_str), Some("wsl.exe"));
    }

    #[test]
    fn driver_for_probe_returns_none_when_unavailable() {
        // On non-Windows, Wsl is always unavailable → driver is None.
        if !cfg!(target_os = "windows") {
            assert!(WslDriver::driver_for_probe(&WslModality::new()).is_none());
        }
    }
}

// ---------------------------------------------------------------------------
// M4 dispatch brief — staged for next session
// ---------------------------------------------------------------------------
//
// What `WslDriver` represents (per ADR-006 M4):
//
//   The probe in `WslModality` answers "is `wsl.exe` reachable on this
//   Windows host?". The *driver* answers "spawn `wsl.exe -e <distro>`,
//   tunnel capture/input through the WSL/Linux distro, and shut it down
//   on App drop".
//
//   M4 is the smallest of the four driver slices because Windows hosts
//   already have `wsl.exe` available (no third-party binary to ship).
//   The tunnel is just a TCP/IPC socket on the WSL side and a Win32
//   stdin/stdout pair on the Windows side — no QEMU/firecracker setup.
//
// Concretely, when a user invokes `playcua --modality wsl screenshot`
// on Windows, the App construction looks up `ModalityRegistry::select(Wsl)`,
// gets a `WslDriver` back, and:
//   - `capture.screenshot()` routes through the WSL X11 socket
//                              (or a VNC bridge for headless distros)
//   - `input.type/key/tap` routes through the same tunnel
//   - the `native` fallback path is replaced entirely
//
// The struct below is intentionally skeletal — the next session fills in:
//
//   1. `spawn()`             — `tokio::process::Command::new("wsl.exe")` with
//                              flags:
//                                - `~`-quoting for paths to handle Windows
//                                  UNC → WSL /mnt/c/... translation
//                                - `--distribution <distro>` from
//                                  `PLAYCUA_WSL_DISTRO` env var (default Ubuntu)
//
//   2. `tunnel()`            — wraps `Child` stdio in a JSON-RPC client that
//                              speaks to the WSL side. Reuses the existing
//                              `Dispatcher` codec in `native/src/dispatch.rs`.
//                              The tunnel lives behind a `RwLock<Option<Tunnel>>`
//                              so the first method-call pays the spawn cost.
//
//   3. `shutdown()`          — graceful child kill on App drop; sends
//                              SIGTERM first, SIGKILL after 5 s. Implemented
//                              in `Drop` for WslDriver.
//
//   4. tests                — hermetic test using a fake `wsl.exe` shell
//                             script in `native/tests/fixtures/fake-wsl.sh`
//                             that echoes the JSON-RPC envelope back. On
//                             non-Windows CI runners this test is #[cfg]'d
//                             to skip (the WSL modality is Windows-only).
//
// Why a skeleton now and not a full impl: cargo test --workspace exceeds
// the 5-minute shell tool timeout on this machine, so anything requiring
// full test evidence belongs in the next session. The skeleton compiles
// and exercises the API shape via the unit tests above — those run in
// well under a second. The Windows-only target compile is verified via
// `cargo check --target x86_64-pc-windows-msvc` from PR #132.

/// Lazy spawn-and-tunnel handle for `wsl.exe`. Windows-only at runtime;
/// compiles on all targets so unit tests can exercise the API shape.
pub struct WslDriver;

impl WslDriver {
    /// Construct a driver. Does not spawn.
    pub fn new() -> Self {
        Self
    }

    /// If WSL is available on this Windows host, return a driver; otherwise None.
    pub fn driver_for_probe(m: &WslModality) -> Option<Self> {
        if m.is_available() {
            Some(Self)
        } else {
            None
        }
    }

    /// The argv head that `spawn()` will eventually exec. Exposed now so
    /// tests can verify the binary selection without spawning.
    pub fn spawn_argv(&self) -> Vec<String> {
        vec!["wsl.exe".to_string()]
    }
}
