//! App layer — dependency injection wiring.
//!
//! Selects the correct adapter implementations based on the compile-time
//! target OS and wires them into a Dispatcher.

use std::sync::Arc;

use crate::adapters::analysis_adapter::NativeAnalysisAdapter;
use crate::adapters::process_adapter::NativeProcessAdapter;
use crate::ipc::dispatcher::Dispatcher;
use crate::ports::{AnalysisPort, CapturePort, InputPort, ProcessPort, WindowPort};

/// The fully-wired application ready to serve IPC requests.
pub struct App {
    pub dispatcher: Dispatcher,
}

impl App {
    /// Construct the application, selecting platform adapters at compile time.
    pub fn build() -> Self {
        let capture: Arc<dyn CapturePort> = build_capture();
        let input: Arc<dyn InputPort> = build_input();
        let windows: Arc<dyn WindowPort> = build_window();
        let process: Arc<dyn ProcessPort> = Arc::new(NativeProcessAdapter::new());
        let analysis: Arc<dyn AnalysisPort> = Arc::new(NativeAnalysisAdapter::new());

        App {
            dispatcher: Dispatcher::new(capture, input, windows, process, analysis),
        }
    }
}

// ---------------------------------------------------------------------------
// Platform adapter selection
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn build_capture() -> Arc<dyn CapturePort> {
    use crate::adapters::windows::wgc::WgcCapture;
    use crate::adapters::xcap::XcapCapture;
    match WgcCapture::new() {
        Ok(wgc) => {
            tracing::info!("Capture: Windows Graphics Capture (primary)");
            Arc::new(wgc)
        }
        Err(e) => {
            tracing::warn!("WGC unavailable ({}), falling back to xcap", e);
            Arc::new(XcapCapture::new())
        }
    }
}

#[cfg(target_os = "linux")]
fn build_capture() -> Arc<dyn CapturePort> {
    use crate::adapters::linux::x11capture::X11Capture;
    tracing::info!("Capture: x11/xcap");
    Arc::new(X11Capture::new())
}

#[cfg(target_os = "macos")]
fn build_capture() -> Arc<dyn CapturePort> {
    use crate::adapters::macos::cgcapture::CGCapture;
    tracing::info!("Capture: CoreGraphics/xcap");
    Arc::new(CGCapture::new())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn build_capture() -> Arc<dyn CapturePort> {
    use crate::adapters::xcap::XcapCapture;
    tracing::warn!("Capture: xcap fallback (unknown platform)");
    Arc::new(XcapCapture::new())
}

// -----------

#[cfg(target_os = "windows")]
fn build_input() -> Arc<dyn InputPort> {
    use crate::adapters::windows::sendinput::SendInputAdapter;
    tracing::info!("Input: Windows SendInput/PostMessage");
    Arc::new(SendInputAdapter::new())
}

#[cfg(target_os = "linux")]
fn build_input() -> Arc<dyn InputPort> {
    use crate::adapters::linux::uinput::UinputAdapter;
    tracing::info!("Input: Linux uinput/enigo");
    Arc::new(UinputAdapter::new())
}

#[cfg(target_os = "macos")]
fn build_input() -> Arc<dyn InputPort> {
    use crate::adapters::macos::cgevent::CGEventAdapter;
    tracing::info!("Input: macOS CGEvent/enigo");
    Arc::new(CGEventAdapter::new())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn build_input() -> Arc<dyn InputPort> {
    use crate::adapters::enigo::EnigoInput;
    tracing::warn!("Input: enigo fallback (unknown platform)");
    Arc::new(EnigoInput::new())
}

// -----------

#[cfg(target_os = "windows")]
fn build_window() -> Arc<dyn WindowPort> {
    use crate::adapters::windows::enumwin::EnumWindowsAdapter;
    tracing::info!("Windows: Win32 EnumWindows");
    Arc::new(EnumWindowsAdapter::new())
}

#[cfg(target_os = "linux")]
fn build_window() -> Arc<dyn WindowPort> {
    use crate::adapters::linux::ewmh::EwmhAdapter;
    tracing::info!("Windows: Linux EWMH/xcap");
    Arc::new(EwmhAdapter::new())
}

#[cfg(target_os = "macos")]
fn build_window() -> Arc<dyn WindowPort> {
    use crate::adapters::macos::nsworkspace::NSWorkspaceAdapter;
    tracing::info!("Windows: macOS NSWorkspace/xcap");
    Arc::new(NSWorkspaceAdapter::new())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn build_window() -> Arc<dyn WindowPort> {
    // Minimal stub that always returns empty list.
    use crate::domain::window::{WindowError, WindowFilter, WindowInfo};
    use crate::ports::WindowPort;
    use async_trait::async_trait;

    struct NoopWindowAdapter;
    #[async_trait]
    impl WindowPort for NoopWindowAdapter {
        async fn list_windows(&self) -> Result<Vec<WindowInfo>, WindowError> { Ok(vec![]) }
        async fn find_window(&self, _: WindowFilter) -> Result<Option<WindowInfo>, WindowError> { Ok(None) }
        async fn focus_window(&self, _: usize) -> Result<(), WindowError> {
            Err(WindowError::Failed("not supported on this platform".to_string()))
        }
    }
    tracing::warn!("Windows: no-op adapter (unknown platform)");
    Arc::new(NoopWindowAdapter)
}
