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
}
