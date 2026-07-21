//! Per-port modality dispatch — wires selected modality into App ports.
//!
//! ADR-006 M2: when `--modality sandbox` is selected, process lifecycle
//! routes through [`WireSandboxAdapter`] / [`SandboxDriver`]. Capture,
//! input, and window ports deliberately do **not** fall back to native
//! host adapters (that would leak host OS calls into the sandbox path).
//! Until the stdio JSON-RPC bridge (`playcua-bridge`) is connected, those
//! ports fail loud with an actionable error.
//!
//! Native modality keeps the existing platform adapters unchanged.

use std::sync::Arc;

use async_trait::async_trait;

use crate::adapters::analysis_adapter::NativeAnalysisAdapter;
use crate::adapters::process_adapter::NativeProcessAdapter;
use crate::adapters::sandbox::WireSandboxAdapter;
use crate::domain::capture::{CaptureError, Frame};
use crate::domain::input::{InputError, Key, KeyAction, MouseEvent};
use crate::domain::process::{ProcessError, ProcessHandle, ProcessStatus};
use crate::domain::sandbox::SandboxSpec;
use crate::domain::window::{WindowError, WindowFilter, WindowInfo};
use crate::modality::registry::SelectedModality;
use crate::modality::ModalityKind;
use crate::ports::sandbox::Sandbox;
use crate::ports::{AnalysisPort, CapturePort, InputPort, ProcessPort, WindowPort};

/// Bundle of port trait objects selected for a modality.
pub struct PortBundle {
    pub capture: Arc<dyn CapturePort>,
    pub input: Arc<dyn InputPort>,
    pub windows: Arc<dyn WindowPort>,
    pub process: Arc<dyn ProcessPort>,
    pub analysis: Arc<dyn AnalysisPort>,
    /// Shared sandbox port when modality is Sandbox (also used by process).
    pub sandbox: Option<Arc<dyn Sandbox>>,
}

/// Build the port bundle for `selected`. Native uses platform adapters;
/// Sandbox uses real driver-backed process dispatch + fail-loud I/O ports.
pub fn build_ports(
    selected: &SelectedModality,
    native_capture: Arc<dyn CapturePort>,
    native_input: Arc<dyn InputPort>,
    native_windows: Arc<dyn WindowPort>,
) -> PortBundle {
    match selected.kind {
        ModalityKind::Native => PortBundle {
            capture: native_capture,
            input: native_input,
            windows: native_windows,
            process: Arc::new(NativeProcessAdapter::new()),
            analysis: Arc::new(NativeAnalysisAdapter::new()),
            sandbox: None,
        },
        ModalityKind::Sandbox => build_sandbox_ports(selected),
        other => {
            // Nvms / Wsl / Container drivers exist but are not yet wired into
            // per-port dispatch. Fail loud on process.launch rather than
            // silently executing on the host (would violate modality isolation).
            tracing::warn!(
                kind = %other,
                "modality selected but per-port dispatch not wired; \
                 process/capture/input fail-loud until M3–M5 dispatch lands"
            );
            let detail = format!(
                "modality `{other}` has a driver API but no per-port dispatch yet \
                 (ADR-006); use --modality native or sandbox"
            );
            PortBundle {
                capture: Arc::new(FailLoudCapture {
                    reason: detail.clone(),
                }),
                input: Arc::new(FailLoudInput {
                    reason: detail.clone(),
                }),
                windows: Arc::new(FailLoudWindow {
                    reason: detail.clone(),
                }),
                process: Arc::new(FailLoudProcess { reason: detail }),
                analysis: Arc::new(NativeAnalysisAdapter::new()),
                sandbox: None,
            }
        }
    }
}

fn build_sandbox_ports(selected: &SelectedModality) -> PortBundle {
    if !selected.available {
        let reason = format!(
            "sandbox modality selected but unavailable ({}); install \
             firejail/sandbox-exec/runsc or set PLAYCUA_SANDBOX_BACKEND=direct",
            selected.detail
        );
        tracing::error!(%reason, "refusing native fallback for sandbox modality");
        return PortBundle {
            capture: Arc::new(FailLoudCapture {
                reason: reason.clone(),
            }),
            input: Arc::new(FailLoudInput {
                reason: reason.clone(),
            }),
            windows: Arc::new(FailLoudWindow {
                reason: reason.clone(),
            }),
            process: Arc::new(FailLoudProcess { reason }),
            analysis: Arc::new(NativeAnalysisAdapter::new()),
            sandbox: None,
        };
    }

    let sandbox: Arc<dyn Sandbox> = Arc::new(WireSandboxAdapter::new());
    let process: Arc<dyn ProcessPort> = Arc::new(SandboxProcessAdapter {
        sandbox: Arc::clone(&sandbox),
    });
    let tunnel_reason = format!(
        "sandbox modality ({}): capture/input/window require the stdio \
         JSON-RPC tunnel to playcua-bridge (not connected in this slice); \
         process.launch is routed through SandboxDriver",
        selected.detail
    );
    PortBundle {
        capture: Arc::new(FailLoudCapture {
            reason: tunnel_reason.clone(),
        }),
        input: Arc::new(FailLoudInput {
            reason: tunnel_reason.clone(),
        }),
        windows: Arc::new(FailLoudWindow {
            reason: tunnel_reason,
        }),
        process,
        analysis: Arc::new(NativeAnalysisAdapter::new()),
        sandbox: Some(sandbox),
    }
}

/// ProcessPort that forwards launch/kill/status through the Sandbox port.
struct SandboxProcessAdapter {
    sandbox: Arc<dyn Sandbox>,
}

#[async_trait]
impl ProcessPort for SandboxProcessAdapter {
    async fn launch(&self, handle: ProcessHandle) -> Result<u32, ProcessError> {
        let spec = SandboxSpec {
            command: handle.path.clone(),
            args: handle.args.clone(),
        };
        let h = self
            .sandbox
            .spawn(&spec)
            .await
            .map_err(|e| ProcessError::LaunchFailed(e.to_string()))?;
        h.id.parse::<u32>()
            .map_err(|e| ProcessError::LaunchFailed(format!("sandbox pid parse: {e}")))
    }

    async fn kill(&self, pid: u32) -> Result<(), ProcessError> {
        use crate::domain::sandbox::SandboxHandle;
        self.sandbox
            .kill(&SandboxHandle {
                id: pid.to_string(),
            })
            .await
            .map_err(|e| match e {
                crate::domain::sandbox::SandboxError::NotFound(_) => ProcessError::NotFound(pid),
                other => ProcessError::KillFailed(other.to_string()),
            })
    }

    async fn status(&self, pid: u32) -> Result<ProcessStatus, ProcessError> {
        use crate::domain::sandbox::SandboxHandle;
        let st = self
            .sandbox
            .status(&SandboxHandle {
                id: pid.to_string(),
            })
            .await
            .map_err(|e| match e {
                crate::domain::sandbox::SandboxError::NotFound(_) => ProcessError::NotFound(pid),
                other => ProcessError::StatusFailed(other.to_string()),
            })?;
        Ok(ProcessStatus {
            running: st.running,
            exit_code: st.exit_code,
        })
    }
}

struct FailLoudProcess {
    reason: String,
}

#[async_trait]
impl ProcessPort for FailLoudProcess {
    async fn launch(&self, _: ProcessHandle) -> Result<u32, ProcessError> {
        Err(ProcessError::LaunchFailed(self.reason.clone()))
    }
    async fn kill(&self, pid: u32) -> Result<(), ProcessError> {
        Err(ProcessError::KillFailed(format!(
            "{} (pid={pid})",
            self.reason
        )))
    }
    async fn status(&self, pid: u32) -> Result<ProcessStatus, ProcessError> {
        Err(ProcessError::StatusFailed(format!(
            "{} (pid={pid})",
            self.reason
        )))
    }
}

struct FailLoudCapture {
    reason: String,
}

#[async_trait]
impl CapturePort for FailLoudCapture {
    async fn capture_display(&self, _: u32) -> Result<Frame, CaptureError> {
        Err(CaptureError::CaptureFailed(self.reason.clone()))
    }
    async fn capture_window(&self, _: Option<&str>) -> Result<Frame, CaptureError> {
        Err(CaptureError::CaptureFailed(self.reason.clone()))
    }
}

struct FailLoudInput {
    reason: String,
}

#[async_trait]
impl InputPort for FailLoudInput {
    async fn key_event(&self, _: Key, _: KeyAction) -> Result<(), InputError> {
        Err(InputError::InjectionFailed(self.reason.clone()))
    }
    async fn type_text(&self, _: &str) -> Result<(), InputError> {
        Err(InputError::InjectionFailed(self.reason.clone()))
    }
    async fn mouse_event(&self, _: MouseEvent) -> Result<(), InputError> {
        Err(InputError::InjectionFailed(self.reason.clone()))
    }
}

struct FailLoudWindow {
    reason: String,
}

#[async_trait]
impl WindowPort for FailLoudWindow {
    async fn list_windows(&self) -> Result<Vec<WindowInfo>, WindowError> {
        Err(WindowError::Failed(self.reason.clone()))
    }
    async fn find_window(&self, _: WindowFilter) -> Result<Option<WindowInfo>, WindowError> {
        Err(WindowError::Failed(self.reason.clone()))
    }
    async fn focus_window(&self, _: usize) -> Result<(), WindowError> {
        Err(WindowError::Failed(self.reason.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modality::ModalityKind;

    fn selected(kind: ModalityKind, available: bool, detail: &str) -> SelectedModality {
        SelectedModality {
            kind,
            describe: "test",
            detail: detail.to_string(),
            available,
        }
    }

    struct NoopCapture;
    #[async_trait]
    impl CapturePort for NoopCapture {
        async fn capture_display(&self, _: u32) -> Result<Frame, CaptureError> {
            Err(CaptureError::CaptureFailed("noop".into()))
        }
        async fn capture_window(&self, _: Option<&str>) -> Result<Frame, CaptureError> {
            Err(CaptureError::CaptureFailed("noop".into()))
        }
    }
    struct NoopInput;
    #[async_trait]
    impl InputPort for NoopInput {
        async fn key_event(&self, _: Key, _: KeyAction) -> Result<(), InputError> {
            Ok(())
        }
        async fn type_text(&self, _: &str) -> Result<(), InputError> {
            Ok(())
        }
        async fn mouse_event(&self, _: MouseEvent) -> Result<(), InputError> {
            Ok(())
        }
    }
    struct NoopWindow;
    #[async_trait]
    impl WindowPort for NoopWindow {
        async fn list_windows(&self) -> Result<Vec<WindowInfo>, WindowError> {
            Ok(vec![])
        }
        async fn find_window(&self, _: WindowFilter) -> Result<Option<WindowInfo>, WindowError> {
            Ok(None)
        }
        async fn focus_window(&self, _: usize) -> Result<(), WindowError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn sandbox_unavailable_fails_loud_on_process_launch() {
        let ports = build_ports(
            &selected(ModalityKind::Sandbox, false, "no backend"),
            Arc::new(NoopCapture),
            Arc::new(NoopInput),
            Arc::new(NoopWindow),
        );
        let err = ports
            .process
            .launch(ProcessHandle::new("echo"))
            .await
            .expect_err("must fail loud");
        assert!(err.to_string().contains("unavailable"));
    }

    #[tokio::test]
    async fn sandbox_available_routes_process_through_driver() {
        let _guard = crate::modality::sandbox::SANDBOX_ENV_LOCK
            .lock()
            .expect("sandbox env lock");
        let prev = std::env::var("PLAYCUA_SANDBOX_BACKEND").ok();
        std::env::set_var("PLAYCUA_SANDBOX_BACKEND", "direct");
        let ports = build_ports(
            &selected(ModalityKind::Sandbox, true, "backend=direct"),
            Arc::new(NoopCapture),
            Arc::new(NoopInput),
            Arc::new(NoopWindow),
        );
        assert!(ports.sandbox.is_some());
        #[cfg(unix)]
        let handle = ProcessHandle::new("sleep").with_args(vec!["30".into()]);
        #[cfg(windows)]
        let handle =
            ProcessHandle::new("cmd").with_args(vec!["/C".into(), "ping -n 30 127.0.0.1 >NUL".into()]);
        let pid = ports
            .process
            .launch(handle)
            .await
            .expect("sandbox process launch");
        assert!(pid > 0);
        ports.process.kill(pid).await.expect("kill");
        match prev {
            Some(v) => std::env::set_var("PLAYCUA_SANDBOX_BACKEND", v),
            None => std::env::remove_var("PLAYCUA_SANDBOX_BACKEND"),
        }
    }

    #[tokio::test]
    async fn sandbox_capture_does_not_silently_use_native() {
        let ports = build_ports(
            &selected(ModalityKind::Sandbox, true, "backend=direct"),
            Arc::new(NoopCapture),
            Arc::new(NoopInput),
            Arc::new(NoopWindow),
        );
        let err = ports
            .capture
            .capture_display(0)
            .await
            .expect_err("must not use native capture");
        assert!(err.to_string().contains("tunnel") || err.to_string().contains("bridge"));
    }
}
