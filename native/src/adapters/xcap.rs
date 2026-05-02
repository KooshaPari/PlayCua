//! XcapCapture adapter — cross-platform fallback capture using the xcap crate.
//! Implements CapturePort for all platforms when a primary adapter is unavailable.

use crate::domain::capture::{CaptureError, Frame};
use crate::ports::CapturePort;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::DynamicImage;
use tracing::instrument;

pub struct XcapCapture;

impl XcapCapture {
    pub fn new() -> Self {
        Self
    }
}

impl Default for XcapCapture {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapturePort for XcapCapture {
    #[instrument(name = "xcap.capture_display", skip(self))]
    async fn capture_display(&self, monitor: u32) -> Result<Frame, CaptureError> {
        let monitor_idx = monitor as usize;
        tokio::task::spawn_blocking(move || -> Result<Frame, CaptureError> {
            let monitors = xcap::Monitor::all()
                .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
            let mon = monitors
                .into_iter()
                .nth(monitor_idx)
                .ok_or_else(|| CaptureError::WindowNotFound(format!("monitor index {}", monitor_idx)))?;
            let img = mon
                .capture_image()
                .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
            encode_xcap_image(img.width(), img.height(), img.into_raw())
        })
        .await
        .map_err(|e| CaptureError::CaptureFailed(format!("spawn_blocking panic: {e}")))?
    }

    #[instrument(name = "xcap.capture_window", skip(self))]
    async fn capture_window(&self, title: Option<&str>) -> Result<Frame, CaptureError> {
        let title_owned = title.map(|t| t.to_string());
        tokio::task::spawn_blocking(move || -> Result<Frame, CaptureError> {
            let title = title_owned.ok_or_else(|| {
                CaptureError::WindowNotFound("no title provided for window capture".to_string())
            })?;
            let windows = xcap::Window::all()
                .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
            let window = windows
                .into_iter()
                .find(|w| w.title().is_ok_and(|t| t.to_lowercase().contains(&title.to_lowercase())))
                .ok_or_else(|| CaptureError::WindowNotFound(title.clone()))?;
            let img = window
                .capture_image()
                .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
            encode_xcap_image(img.width(), img.height(), img.into_raw())
        })
        .await
        .map_err(|e| CaptureError::CaptureFailed(format!("spawn_blocking panic: {e}")))?
    }
}

fn encode_xcap_image(width: u32, height: u32, raw: Vec<u8>) -> Result<Frame, CaptureError> {
    let img_buf = image::ImageBuffer::from_raw(width, height, raw)
        .ok_or_else(|| CaptureError::EncodeFailed("ImageBuffer construction failed".to_string()))?;
    let dyn_img = DynamicImage::ImageRgba8(img_buf);
    encode_png_frame(&dyn_img)
}

pub(crate) fn encode_png_frame(img: &DynamicImage) -> Result<Frame, CaptureError> {
    let width = img.width();
    let height = img.height();
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| CaptureError::EncodeFailed(e.to_string()))?;
    let data = BASE64.encode(&buf);
    Ok(Frame { data, width, height })
}
