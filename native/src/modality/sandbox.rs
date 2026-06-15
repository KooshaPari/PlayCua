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
enum SandboxBackend {
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
    fn binary(self) -> &'static str {
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
}
