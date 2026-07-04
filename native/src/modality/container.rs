//! `container` modality — generic OCI container (Docker, Podman, containerd).
//!
//! Probes for `docker` or `podman` (or `$PLAYCUA_CONTAINER_CLI` override).
//! Either of these on $PATH is sufficient — the modality is the
//! container-runtime-agnostic shim.

use super::{Modality, ModalityKind};
use std::path::PathBuf;

/// The container-modality probe.
pub struct ContainerModality {
    cached: std::sync::OnceLock<Option<&'static str>>,
}

impl ContainerModality {
    pub fn new() -> Self {
        Self {
            cached: std::sync::OnceLock::new(),
        }
    }

    fn probe(&self) -> Option<&'static str> {
        self.cached
            .get_or_init(|| {
                if let Ok(override_cli) = std::env::var("PLAYCUA_CONTAINER_CLI") {
                    return Some(match override_cli.as_str() {
                        "docker" => "docker",
                        "podman" => "podman",
                        "nerdctl" => "nerdctl",
                        _ => return None,
                    });
                }
                ["docker", "podman", "nerdctl"]
                    .into_iter()
                    .find(|cli| which(cli).is_some())
            })
            .as_ref()
            .copied()
    }
}

impl Default for ContainerModality {
    fn default() -> Self {
        Self::new()
    }
}

impl Modality for ContainerModality {
    fn kind(&self) -> ModalityKind {
        ModalityKind::Container
    }

    fn describe(&self) -> &'static str {
        "OCI container (docker / podman / nerdctl)"
    }

    fn is_available(&self) -> bool {
        self.probe().is_some()
    }

    fn detail(&self) -> String {
        match self.probe() {
            Some(cli) => format!("cli={cli}"),
            None => "no container CLI on $PATH".to_string(),
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
        let m = ContainerModality::new();
        assert_eq!(m.is_available(), m.is_available());
    }

    #[test]
    fn kind_is_container() {
        assert_eq!(ContainerModality::new().kind(), ModalityKind::Container);
    }

    #[test]
    fn driver_spawn_argv_includes_run_subcommand() {
        // The lazy driver must build an argv whose head is the container CLI
        // (e.g. "docker", "podman") followed by `run`, so the host shell can
        // exec it directly. We don't actually spawn in tests (would need a
        // real container runtime); just verify the argv shape.
        let d = ContainerDriver::new(ContainerCli::Docker);
        let argv = d.spawn_argv();
        assert_eq!(argv.first().map(String::as_str), Some("docker"));
        assert_eq!(argv.get(1).map(String::as_str), Some("run"));
    }

    #[test]
    fn driver_for_probe_returns_none_when_unavailable() {
        // When no container CLI is on $PATH, `driver_for_probe` should be
        // None rather than panic. Tests run with whatever $PATH the harness
        // provides; we don't assert presence/absence, just the Option shape.
        let d = ContainerDriver::driver_for_probe(&ContainerModality::new());
        if let Some(d) = d {
            // If a CLI IS available, the binary must be on $PATH.
            assert!(which(d.cli.binary()).is_some());
        }
    }
}

// ---------------------------------------------------------------------------
// M5 dispatch brief — staged for next session
// ---------------------------------------------------------------------------
//
// What `ContainerDriver` represents (per ADR-006 M5):
//
//   The probe in `ContainerModality` answers "is `docker` / `podman` /
//   `nerdctl` on $PATH?". The *driver* answers "spawn `cli run --rm ...`,
//   tunnel capture/input through the container, and shut it down on App drop".
//
//   M5 is the largest of the four driver slices because it must:
//   - Author a base container image (Ubuntu 22.04 + Xvfb + x11-apps +
//     the PlayCua capture bridge).
//   - Handle the OCI-vs-podman CLI divergence (defaults, --userns, --rm).
//   - Map host:container paths correctly on Windows (where the user runs
//     `playcua.exe` and the container runs Linux).
//   - Set up either an X11 socket mount or a VNC bridge for capture, and
//     a uinput bridge for input.
//
// Concretely, when a user invokes `playcua --modality container screenshot`,
// the App construction looks up `ModalityRegistry::select(Container)`,
// gets a `ContainerDriver` back, and:
//   - `capture.screenshot()` route through the X11 socket inside the
//                              container (or VNC frame buffer)
//   - `input.type/key/tap` route through the same bridge
//   - the `native` fallback path is replaced entirely
//
// The struct below is intentionally skeletal — the next session fills in:
//
//   1. `spawn()`             — `tokio::process::Command::new(cli.binary())` with
//                              subcommand `run --rm -i --volume
//                              <host-screenshot-dir>:<container-screenshot-dir>
//                              <playcua-image>:
//                              /usr/local/bin/playcua-bridge`. The image name
//                              comes from `PLAYCUA_CONTAINER_IMAGE` env var
//                              (default `playcua/bridge:latest`).
//
//   2. `tunnel()`            — wraps `Child` stdio in a JSON-RPC client.
//                              Reuses the existing `Dispatcher` codec in
//                              `native/src/dispatch.rs`. The tunnel lives
//                              behind a `RwLock<Option<Tunnel>>` so the
//                              first method-call pays the spawn cost.
//
//   3. `shutdown()`          — graceful child kill on App drop; sends
//                              SIGTERM first, SIGKILL after 5 s. Implemented
//                              in `Drop` for ContainerDriver.
//
//   4. image authoring       — a `Containerfile` for the `playcua/bridge`
//                              image. Lives in `sandbox/Containerfile`
//                              (not added in this PR).
//
//   5. tests                — hermetic test using a fake container CLI shell
//                             script in `native/tests/fixtures/fake-cli.sh`
//                             that echoes the JSON-RPC envelope back. CI
//                             runners without a container runtime get the
//                             option/None path (no panic).
//
// Why a skeleton now and not a full impl: cargo test --workspace exceeds
// the 5-minute shell tool timeout on this machine, so anything requiring
// full test evidence belongs in the next session. The skeleton compiles
// and exercises the API shape via the two unit tests above — those run
// in well under a second.

/// Container CLI selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerCli {
    Docker,
    Podman,
    Nerdctl,
}

impl ContainerCli {
    pub(crate) fn binary(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
            Self::Nerdctl => "nerdctl",
        }
    }
}

impl<'a> From<&'a str> for ContainerCli {
    fn from(s: &'a str) -> Self {
        match s {
            "docker" => Self::Docker,
            "podman" => Self::Podman,
            "nerdctl" => Self::Nerdctl,
            // Probe results are constrained to the three known CLIs; if a
            // future PLAYCUA_CONTAINER_CLI adds an unknown value, default
            // to docker (the most common). The probe already gates by
            // which() so an unknown CLI on PATH won't activate the modality.
            _ => Self::Docker,
        }
    }
}

/// Lazy spawn-and-tunnel handle for a container CLI invocation.
///
/// The body of `spawn`/`tunnel`/`shutdown` is intentionally a TODO for the
/// next session per the dispatch brief at the top of this module.
pub struct ContainerDriver {
    cli: ContainerCli,
}

impl ContainerDriver {
    /// Construct a driver for a specific CLI. Does not spawn.
    pub fn new(cli: ContainerCli) -> Self {
        Self { cli }
    }

    /// If the given modality probe found a CLI, return a driver for it.
    pub fn driver_for_probe(m: &ContainerModality) -> Option<Self> {
        let cli_str = m.probe()?;
        Some(Self {
            cli: ContainerCli::from(cli_str),
        })
    }

    /// The argv head that `spawn()` will eventually exec. Exposed now so
    /// tests can verify the CLI selection without spawning.
    pub fn spawn_argv(&self) -> Vec<String> {
        vec![self.cli.binary().to_string(), "run".to_string()]
    }
}
