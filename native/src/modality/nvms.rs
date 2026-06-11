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
}
