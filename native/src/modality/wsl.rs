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
}
