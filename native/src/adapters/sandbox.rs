//! Production wire adapter for the Sandbox port.
//!
//! Spawns via [`SandboxDriver`] after probing [`SandboxModality`]. Fail-loud
//! when no backend is available (no silent stub success).
//!
//! Hermetic / CI: set `PLAYCUA_SANDBOX_BACKEND=direct` so the guest command
//! runs without a host sandbox binary on `$PATH`.

use crate::domain::sandbox::{SandboxError, SandboxHandle, SandboxSpec, SandboxStatus};
use crate::modality::sandbox::{SandboxDriver, SandboxModality};
use crate::ports::sandbox::Sandbox;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Production wire adapter — dispatches through [`SandboxDriver`].
pub struct WireSandboxAdapter {
    live: Arc<Mutex<HashMap<String, SandboxDriver>>>,
}

impl WireSandboxAdapter {
    pub fn new() -> Self {
        Self {
            live: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for WireSandboxAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Sandbox for WireSandboxAdapter {
    async fn spawn(&self, spec: &SandboxSpec) -> Result<SandboxHandle, SandboxError> {
        let modality = SandboxModality::new();
        let mut driver = SandboxDriver::driver_for_probe(&modality).ok_or_else(|| {
            SandboxError::SpawnFailed(
                "no sandbox backend on $PATH; install firejail/sandbox-exec/runsc \
                 or set PLAYCUA_SANDBOX_BACKEND=direct for hermetic spawn"
                    .into(),
            )
        })?;
        driver
            .spawn_guest(&spec.command, &spec.args)
            .await
            .map_err(|e| SandboxError::SpawnFailed(format!("{}: {e}", spec.command)))?;
        let id = driver
            .child_id()
            .ok_or_else(|| SandboxError::SpawnFailed("child process missing pid".into()))?
            .to_string();
        self.live.lock().await.insert(id.clone(), driver);
        Ok(SandboxHandle { id })
    }

    async fn status(&self, handle: &SandboxHandle) -> Result<SandboxStatus, SandboxError> {
        let mut live = self.live.lock().await;
        let driver = live
            .get_mut(&handle.id)
            .ok_or_else(|| SandboxError::NotFound(handle.id.clone()))?;
        match driver.try_status() {
            Ok(Some((running, exit_code))) => Ok(SandboxStatus {
                running,
                exit_code,
            }),
            Ok(None) => Err(SandboxError::StatusFailed(format!(
                "driver for {} has no child",
                handle.id
            ))),
            Err(e) => Err(SandboxError::StatusFailed(e.to_string())),
        }
    }

    async fn kill(&self, handle: &SandboxHandle) -> Result<(), SandboxError> {
        let mut live = self.live.lock().await;
        let mut driver = live
            .remove(&handle.id)
            .ok_or_else(|| SandboxError::NotFound(handle.id.clone()))?;
        driver
            .shutdown()
            .await
            .map_err(|e| SandboxError::KillFailed(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modality::sandbox::SANDBOX_ENV_LOCK;

    /// Wire adapter fails loud when no backend is configured.
    #[tokio::test]
    async fn wire_sandbox_fails_loud_without_backend() {
        let _guard = SANDBOX_ENV_LOCK.lock().expect("sandbox env lock");
        let prev = std::env::var("PLAYCUA_SANDBOX_BACKEND").ok();
        std::env::set_var("PLAYCUA_SANDBOX_BACKEND", "not-a-real-backend");
        let adapter = WireSandboxAdapter::new();
        let spec = SandboxSpec {
            command: "echo".into(),
            args: vec!["test".into()],
        };
        let err = adapter.spawn(&spec).await.expect_err("must fail loud");
        assert!(
            matches!(err, SandboxError::SpawnFailed(_)),
            "expected SpawnFailed, got {err:?}"
        );
        match prev {
            Some(v) => std::env::set_var("PLAYCUA_SANDBOX_BACKEND", v),
            None => std::env::remove_var("PLAYCUA_SANDBOX_BACKEND"),
        }
    }

    /// Wire adapter + Direct backend actually spawns and kills a guest.
    #[tokio::test]
    async fn wire_sandbox_direct_spawn_and_kill() {
        let _guard = SANDBOX_ENV_LOCK.lock().expect("sandbox env lock");
        let prev = std::env::var("PLAYCUA_SANDBOX_BACKEND").ok();
        std::env::set_var("PLAYCUA_SANDBOX_BACKEND", "direct");
        let adapter = WireSandboxAdapter::new();
        #[cfg(unix)]
        let spec = SandboxSpec {
            command: "sleep".into(),
            args: vec!["30".into()],
        };
        #[cfg(windows)]
        let spec = SandboxSpec {
            command: "cmd".into(),
            args: vec!["/C".into(), "ping -n 30 127.0.0.1 >NUL".into()],
        };
        let handle = adapter
            .spawn(&spec)
            .await
            .expect("direct spawn must succeed");
        let status = adapter.status(&handle).await.expect("status");
        assert!(status.running, "child should still be running");
        adapter.kill(&handle).await.expect("kill must succeed");
        let missing = adapter.status(&handle).await;
        assert!(matches!(missing, Err(SandboxError::NotFound(_))));
        match prev {
            Some(v) => std::env::set_var("PLAYCUA_SANDBOX_BACKEND", v),
            None => std::env::remove_var("PLAYCUA_SANDBOX_BACKEND"),
        }
    }
}
