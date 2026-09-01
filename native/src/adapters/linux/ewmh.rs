//! EwmhAdapter — Linux window enumeration via xcap (EWMH/x11rb-backed).
//! Implements WindowPort for Linux.
//!
//! Full x11rb EWMH implementation for window focus is a TODO;
//! enumeration is covered by xcap.

use crate::domain::window::{WindowError, WindowFilter, WindowInfo};
use crate::ports::WindowPort;
use async_trait::async_trait;
use tracing::{instrument, warn};

pub struct EwmhAdapter;

impl EwmhAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EwmhAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WindowPort for EwmhAdapter {
    #[instrument(name = "ewmh.list_windows", skip(self))]
    async fn list_windows(&self) -> Result<Vec<WindowInfo>, WindowError> {
        tokio::task::spawn_blocking(|| -> Result<Vec<WindowInfo>, WindowError> {
            let windows =
                xcap::Window::all().map_err(|e| WindowError::EnumerationFailed(e.to_string()))?;
            windows
                .into_iter()
                .map(|w| {
                    let map_error =
                        |e: xcap::XCapError| WindowError::EnumerationFailed(e.to_string());
                    Ok(WindowInfo {
                        hwnd: w.id().map_err(map_error)? as usize,
                        title: w.title().map_err(map_error)?,
                        pid: w.pid().map_err(map_error)?,
                        x: w.x().map_err(map_error)?,
                        y: w.y().map_err(map_error)?,
                        width: w.width().map_err(map_error)? as i32,
                        height: w.height().map_err(map_error)? as i32,
                        visible: true, // xcap only returns visible windows
                    })
                })
                .collect()
        })
        .await
        .map_err(|e| WindowError::Failed(format!("spawn_blocking panic: {e}")))?
    }

    #[instrument(name = "ewmh.find_window", skip(self))]
    async fn find_window(&self, filter: WindowFilter) -> Result<Option<WindowInfo>, WindowError> {
        let all = self.list_windows().await?;
        let found = all.into_iter().find(|w| {
            let title_match = filter
                .title
                .as_ref()
                .map_or(true, |t| w.title.to_lowercase().contains(&t.to_lowercase()));
            let pid_match = filter.pid.map_or(true, |p| w.pid == p);
            title_match && pid_match
        });
        Ok(found)
    }

    #[instrument(name = "ewmh.focus_window", skip(self))]
    async fn focus_window(&self, _hwnd: usize) -> Result<(), WindowError> {
        // TODO: implement using x11rb _NET_ACTIVE_WINDOW ClientMessage.
        warn!("windows.focus: not yet implemented on Linux (stub)");
        Ok(())
    }
}
