//! Linux window capture via X11 + XCB.

use crate::domain::capture::CaptureError;
use crate::ports::{CapturedFrame, WindowCapturer, WindowDescriptor};

#[cfg(target_os = "linux")]
use xcb::{x, Xid, XidNew};

#[derive(Debug, Default)]
pub struct X11WindowCapturer;

impl X11WindowCapturer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "linux")]
impl WindowCapturer for X11WindowCapturer {
    fn capture(&self, window_id: u64) -> Result<CapturedFrame, CaptureError> {
        let (connection, screen_num) = xcb::Connection::connect(None)
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        let setup = connection.get_setup();
        let screen = setup
            .roots()
            .nth(screen_num as usize)
            .ok_or_else(|| CaptureError::CaptureFailed("X11 screen not found".to_string()))?;
        let window = x::Window::new(window_id as u32);
        let drawable = x::Drawable::Window(window);
        let geometry_cookie = connection.send_request(&x::GetGeometry { drawable });
        let geometry = connection
            .wait_for_reply(geometry_cookie)
            .map_err(|e| CaptureError::CaptureFailed(format!("get_geometry: {e}")))?;

        let image_cookie = connection.send_request(&x::GetImage {
            format: x::ImageFormat::ZPixmap,
            drawable,
            x: 0,
            y: 0,
            width: geometry.width(),
            height: geometry.height(),
            plane_mask: u32::MAX,
        });
        let reply = connection
            .wait_for_reply(image_cookie)
            .map_err(|e| CaptureError::CaptureFailed(format!("get_image: {e}")))?;

        let bits_per_pixel = bits_per_pixel_for_depth(screen.root_depth()).ok_or_else(|| {
            CaptureError::CaptureFailed(format!(
                "unsupported root depth {} for X11 capture",
                screen.root_depth()
            ))
        })?;
        let pixels = decode_zpixmap_bgra(
            reply.data(),
            geometry.width() as u32,
            geometry.height() as u32,
            bits_per_pixel,
        )?;

        Ok(CapturedFrame {
            width: geometry.width() as u32,
            height: geometry.height() as u32,
            pixels,
        })
    }

    fn list_windows(&self) -> Result<Vec<WindowDescriptor>, CaptureError> {
        let (connection, screen_num) = xcb::Connection::connect(None)
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        let setup = connection.get_setup();
        let screen = setup
            .roots()
            .nth(screen_num as usize)
            .ok_or_else(|| CaptureError::CaptureFailed("X11 screen not found".to_string()))?;
        let tree_cookie = connection.send_request(&x::QueryTree {
            window: screen.root(),
        });
        let tree = connection
            .wait_for_reply(tree_cookie)
            .map_err(|e| CaptureError::CaptureFailed(format!("query_tree: {e}")))?;

        tree.children()
            .iter()
            .copied()
            .filter_map(|window| describe_window(&connection, window).transpose())
            .collect()
    }
}

#[cfg(not(target_os = "linux"))]
impl WindowCapturer for X11WindowCapturer {
    fn capture(&self, _window_id: u64) -> Result<CapturedFrame, CaptureError> {
        Err(CaptureError::CaptureFailed(
            "X11WindowCapturer is only available on Linux".to_string(),
        ))
    }

    fn list_windows(&self) -> Result<Vec<WindowDescriptor>, CaptureError> {
        Err(CaptureError::CaptureFailed(
            "X11WindowCapturer is only available on Linux".to_string(),
        ))
    }
}

#[cfg(target_os = "linux")]
fn describe_window(
    connection: &xcb::Connection,
    window: x::Window,
) -> Result<Option<WindowDescriptor>, CaptureError> {
    let geometry_cookie = connection.send_request(&x::GetGeometry {
        drawable: x::Drawable::Window(window),
    });
    let geometry = match connection.wait_for_reply(geometry_cookie) {
        Ok(geometry) => geometry,
        Err(_) => return Ok(None),
    };
    let title = get_window_title(connection, window)?;
    Ok(Some(WindowDescriptor {
        id: u64::from(window.resource_id()),
        title,
        width: geometry.width() as u32,
        height: geometry.height() as u32,
    }))
}

#[cfg(target_os = "linux")]
fn get_window_title(
    connection: &xcb::Connection,
    window: x::Window,
) -> Result<String, CaptureError> {
    let cookie = connection.send_request(&x::GetProperty {
        delete: false,
        window,
        property: x::ATOM_WM_NAME,
        r#type: x::ATOM_STRING,
        long_offset: 0,
        long_length: 1024,
    });
    let reply = connection
        .wait_for_reply(cookie)
        .map_err(|e| CaptureError::CaptureFailed(format!("get_property WM_NAME: {e}")))?;
    Ok(String::from_utf8_lossy(reply.value::<u8>()).into_owned())
}

fn bits_per_pixel_for_depth(depth: u8) -> Option<u8> {
    match depth {
        32 | 24 => Some(32),
        16 => Some(16),
        _ => None,
    }
}

fn decode_zpixmap_bgra(
    data: &[u8],
    width: u32,
    height: u32,
    bits_per_pixel: u8,
) -> Result<Vec<u8>, CaptureError> {
    let bytes_per_pixel = usize::from(bits_per_pixel / 8);
    if bytes_per_pixel == 0 {
        return Err(CaptureError::CaptureFailed(
            "bits_per_pixel must be at least 8".to_string(),
        ));
    }

    let pixel_count = width
        .checked_mul(height)
        .ok_or_else(|| CaptureError::CaptureFailed("frame dimensions overflow".to_string()))?
        as usize;
    let expected_len = pixel_count
        .checked_mul(bytes_per_pixel)
        .ok_or_else(|| CaptureError::CaptureFailed("frame byte count overflow".to_string()))?;

    if data.len() < expected_len {
        return Err(CaptureError::CaptureFailed(format!(
            "X11 image payload too short: expected at least {expected_len} bytes, got {}",
            data.len()
        )));
    }

    let mut pixels = Vec::with_capacity(pixel_count * 4);
    for chunk in data[..expected_len].chunks_exact(bytes_per_pixel) {
        match bits_per_pixel {
            32 => pixels.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]),
            16 => {
                let value = u16::from_ne_bytes([chunk[0], chunk[1]]);
                let red = ((value >> 11) & 0x1f) as u8;
                let green = ((value >> 5) & 0x3f) as u8;
                let blue = (value & 0x1f) as u8;
                pixels.extend_from_slice(&[
                    (red << 3) | (red >> 2),
                    (green << 2) | (green >> 4),
                    (blue << 3) | (blue >> 2),
                    0xff,
                ]);
            }
            _ => {
                return Err(CaptureError::CaptureFailed(format!(
                    "unsupported bits_per_pixel {bits_per_pixel}"
                )));
            }
        }
    }

    Ok(pixels)
}

#[cfg(test)]
mod tests {
    use super::{bits_per_pixel_for_depth, decode_zpixmap_bgra};

    #[test]
    fn maps_common_x11_depths_to_bpp() {
        assert_eq!(bits_per_pixel_for_depth(24), Some(32));
        assert_eq!(bits_per_pixel_for_depth(16), Some(16));
        assert_eq!(bits_per_pixel_for_depth(8), None);
    }

    #[test]
    fn decodes_32bpp_bgra_pixels_into_rgba() {
        let pixels =
            decode_zpixmap_bgra(&[0x10, 0x20, 0x30, 0x40, 0xaa, 0xbb, 0xcc, 0xdd], 2, 1, 32)
                .expect("32bpp conversion should succeed");

        assert_eq!(pixels, vec![0x30, 0x20, 0x10, 0x40, 0xcc, 0xbb, 0xaa, 0xdd]);
    }
}
