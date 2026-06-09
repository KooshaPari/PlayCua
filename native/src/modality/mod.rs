//! Modality abstraction — pluggable execution environment selection.
//!
//! A **modality** is the runtime environment in which bare-cua-native operates.
//! PlayCua supports five modalities per the NVMSCUA framework (see SPEC.md):
//!
//! - **native**: drive the host OS directly (the only modality with a fully-wired
//!   App in this slice).
//! - **sandbox**: drive a process running inside a sealed sandbox (Windows
//!   Sandbox, Firecracker microVM, gVisor). Selection in this slice is
//!   supported but routing through the sandbox is a follow-up.
//! - **nvms**: drive a process running inside an `nvms`-orchestrated container
//!   (the nanovms repo is the canonical home). The NvmsModality probes for
//!   the `nvms` binary and reports availability.
//! - **wsl**: drive a process running inside WSL (Windows-only). Skeleton.
//! - **container**: drive a process inside a generic OCI container (Docker,
//!   Podman, containerd). Skeleton.
//!
//! ## What the trait actually does
//!
//! The trait in this slice is **observability + selection**, not per-method
//! routing. Each modality reports its kind, a human-readable description, an
//! availability probe, and (optionally) command-line / connection metadata.
//! The dispatcher and port traits are unchanged.
//!
//! Full per-method routing (e.g. "screenshot goes to native, but input.type
//! goes through a sandboxed agent") is intentionally out of scope for this
//! slice — that requires splitting the App construction into per-port
//! modality lookup, which is a larger refactor (tracked in ADR-006).
//!
//! ## Selection precedence
//!
//! 1. `--modality` CLI flag (highest)
//! 2. `BARE_CUA_MODALITY` env var
//! 3. `auto` heuristic (host OS + binary probing)
//! 4. `native` (lowest — always works)

pub mod container;
pub mod native;
pub mod nvms;
pub mod registry;
pub mod sandbox;
pub mod wsl;

use std::fmt;

pub use registry::{ModalityEnv, ModalityRegistry, SelectedModality};

/// The five modalities supported by the NVMSCUA framework.
///
/// Order is meaningful: it doubles as the precedence for `auto` selection
/// (lower index = preferred), with `Native` always last as the
/// always-available fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModalityKind {
    /// Drive the host OS directly. Always available.
    Native = 0,
    /// Windows Sandbox / Firecracker / gVisor / Firejail.
    Sandbox = 1,
    /// `nvms`-orchestrated container (nanovms).
    Nvms = 2,
    /// WSL (Windows Subsystem for Linux, Windows host only).
    Wsl = 3,
    /// Generic OCI container (Docker, Podman, containerd).
    Container = 4,
}

impl ModalityKind {
    /// All known modality kinds, in selection precedence order.
    pub const ALL: &'static [ModalityKind] = &[
        Self::Sandbox,
        Self::Nvms,
        Self::Wsl,
        Self::Container,
        Self::Native,
    ];

    /// Parse from a string. Accepts lowercase, uppercase, and mixed case
    /// (e.g. "Native", "native", "NATIVE" all work).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "native" => Some(Self::Native),
            "sandbox" => Some(Self::Sandbox),
            "nvms" => Some(Self::Nvms),
            "wsl" => Some(Self::Wsl),
            "container" | "docker" | "podman" => Some(Self::Container),
            _ => None,
        }
    }

    /// Stable lowercase identifier (used in env vars and CLI flags).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Sandbox => "sandbox",
            Self::Nvms => "nvms",
            Self::Wsl => "wsl",
            Self::Container => "container",
        }
    }
}

impl fmt::Display for ModalityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A modality implementation: probes for availability and describes itself.
pub trait Modality: Send + Sync {
    /// What kind of modality this is.
    fn kind(&self) -> ModalityKind;

    /// Short human-readable description (e.g. "xcap+enigo on macOS 14.4").
    fn describe(&self) -> &'static str;

    /// Probe whether the modality is currently usable in this environment.
    /// Should be cheap (a single `which` lookup, not a network call).
    fn is_available(&self) -> bool;

    /// Optional extra detail for logs (e.g. probed binary path, version).
    /// Default = empty string.
    fn detail(&self) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_round_trips_all_kinds() {
        for k in ModalityKind::ALL {
            let s = k.as_str();
            assert_eq!(ModalityKind::parse(s), Some(*k), "round-trip failed for {s}");
        }
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!(ModalityKind::parse("NATIVE"), Some(ModalityKind::Native));
        assert_eq!(ModalityKind::parse("NvMs"), Some(ModalityKind::Nvms));
        assert_eq!(ModalityKind::parse("Docker"), Some(ModalityKind::Container));
    }

    #[test]
    fn parse_returns_none_for_unknown() {
        assert_eq!(ModalityKind::parse(""), None);
        assert_eq!(ModalityKind::parse("qemu"), None);
        assert_eq!(ModalityKind::parse("firecracker"), None);
    }

    #[test]
    fn display_matches_as_str() {
        for k in ModalityKind::ALL {
            assert_eq!(format!("{k}"), k.as_str());
        }
    }

    #[test]
    fn all_kinds_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for k in ModalityKind::ALL {
            assert!(seen.insert(*k), "{k} appears twice in ALL");
        }
    }
}
