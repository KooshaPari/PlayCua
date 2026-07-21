//! Guest-side `playcua-bridge` JSON-RPC handlers (screenshot / input.* / windows.*).
//!
//! Speaks the same NDJSON JSON-RPC 2.0 surface that [`super::bridge_client::BridgeClient`]
//! expects. Screenshot and window methods use the same native adapters as host
//! `playcua-native` (xcap / platform capture + window ports). Failures surface as
//! JSON-RPC errors (fail loud) — never a silent empty stub, except when
//! `PLAYCUA_BRIDGE_STUB_SCREENSHOT=1` opts into the hermetic 1×1 PNG for CI.
//!
//! Input methods still ack `{ok:true}` (guest input wiring is a separate slice).
//! `windows.focus` on macOS/Linux matches host dispatcher: honest stub (Ok + warn).

use serde_json::{json, Value};
use tracing::warn;

use crate::app::{native_capture, native_windows};
use crate::domain::capture::{CaptureError, Frame};
use crate::domain::window::WindowFilter;

use super::mod_types::{Request, Response};

/// Canonical 1×1 transparent PNG (base64), shared with FR-001 contract tests.
pub const STUB_PNG_B64: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

/// Env flag: when `1`/`true`, `screenshot` returns the hermetic stub PNG envelope
/// instead of calling guest-OS capture. Documented for CI; default is real capture.
pub const STUB_SCREENSHOT_ENV: &str = "PLAYCUA_BRIDGE_STUB_SCREENSHOT";

/// Whether screenshot is stubbed via [`STUB_SCREENSHOT_ENV`].
pub fn screenshot_capability() -> &'static str {
    if stub_screenshot_enabled() {
        "stub"
    } else {
        "real"
    }
}

/// Window enumeration/focus capability. Always `"real"` on supported platforms
/// (list/find via native adapters); focus may still be an honest platform stub
/// matching host dispatcher semantics (e.g. macOS/Linux).
pub fn windows_capability() -> &'static str {
    "real"
}

fn stub_screenshot_enabled() -> bool {
    match std::env::var(STUB_SCREENSHOT_ENV) {
        Ok(v) => matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"),
        Err(_) => false,
    }
}

/// Dispatch one bridge request. Unknown methods → `-32601` (fail loud).
pub async fn handle_request(req: Request) -> Response {
    let id = req.id.clone();
    let params = req.params.unwrap_or(Value::Null);

    match req.method.as_str() {
        "ping" => Response::ok(
            id,
            json!({
                "ok": true,
                "version": env!("CARGO_PKG_VERSION"),
                "bridge": "playcua-bridge",
                "screenshot": screenshot_capability(),
                "windows": windows_capability(),
            }),
        ),
        "screenshot" => handle_screenshot(id, params).await,
        "input.key" | "input.type" | "input.click" | "input.scroll" | "input.move" => {
            Response::ok(id, json!({ "ok": true }))
        }
        "windows.list" => handle_windows_list(id).await,
        "windows.find" => handle_windows_find(id, params).await,
        "windows.focus" => handle_windows_focus(id, params).await,
        unknown => {
            warn!(method = %unknown, "playcua-bridge unknown method");
            Response::method_not_found(id, unknown)
        }
    }
}

async fn handle_screenshot(id: Value, params: Value) -> Response {
    #[derive(serde::Deserialize, Default)]
    struct P {
        window_title: Option<String>,
        monitor: Option<u32>,
    }
    let p: P = match serde_json::from_value(if params.is_null() {
        json!({})
    } else {
        params
    }) {
        Ok(v) => v,
        Err(e) => return Response::invalid_params(id, e.to_string()),
    };

    if stub_screenshot_enabled() {
        return Response::ok(
            id,
            json!({
                "data": STUB_PNG_B64,
                "width": 1,
                "height": 1,
                "format": "png",
            }),
        );
    }

    let capture = native_capture();
    let result = if let Some(ref title) = p.window_title {
        capture.capture_window(Some(title.as_str())).await
    } else {
        capture.capture_display(p.monitor.unwrap_or(0)).await
    };
    match result {
        Ok(frame) => screenshot_ok(id, frame),
        Err(e) => Response::internal_error(id, format_capture_error(e)),
    }
}

fn screenshot_ok(id: Value, frame: Frame) -> Response {
    Response::ok(
        id,
        json!({
            "data": frame.data,
            "width": frame.width,
            "height": frame.height,
            "format": "png",
        }),
    )
}

fn format_capture_error(e: CaptureError) -> String {
    format!(
        "screenshot failed (guest capture): {e}; \
         grant screen-recording / display access, or set {STUB_SCREENSHOT_ENV}=1 for hermetic stub"
    )
}

async fn handle_windows_list(id: Value) -> Response {
    let windows = native_windows();
    match windows.list_windows().await {
        Ok(wins) => match serde_json::to_value(&wins) {
            Ok(v) => Response::ok(id, v),
            Err(e) => Response::internal_error(id, e.to_string()),
        },
        Err(e) => Response::internal_error(
            id,
            format!("windows.list failed (guest enumeration): {e}"),
        ),
    }
}

async fn handle_windows_find(id: Value, params: Value) -> Response {
    #[derive(serde::Deserialize, Default)]
    struct P {
        title: Option<String>,
        pid: Option<u32>,
    }
    let p: P = match serde_json::from_value(if params.is_null() {
        json!({})
    } else {
        params
    }) {
        Ok(v) => v,
        Err(e) => return Response::invalid_params(id, e.to_string()),
    };
    let filter = WindowFilter {
        title: p.title,
        pid: p.pid,
    };
    let windows = native_windows();
    match windows.find_window(filter).await {
        Ok(Some(w)) => match serde_json::to_value(&w) {
            Ok(v) => Response::ok(id, v),
            Err(e) => Response::internal_error(id, e.to_string()),
        },
        Ok(None) => Response::ok(id, Value::Null),
        Err(e) => Response::internal_error(
            id,
            format!("windows.find failed (guest enumeration): {e}"),
        ),
    }
}

async fn handle_windows_focus(id: Value, params: Value) -> Response {
    #[derive(serde::Deserialize)]
    struct P {
        hwnd: usize,
    }
    let p: P = match serde_json::from_value(params) {
        Ok(v) => v,
        Err(e) => return Response::invalid_params(id, e.to_string()),
    };
    let windows = native_windows();
    match windows.focus_window(p.hwnd).await {
        Ok(()) => Response::ok(id, json!({ "ok": true })),
        Err(e) => Response::internal_error(
            id,
            format!("windows.focus failed: {e}"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::bridge_client::BRIDGE_ENV_LOCK;

    fn req(method: &str, params: Value) -> Request {
        Request {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: method.into(),
            params: if params.is_null() {
                None
            } else {
                Some(params)
            },
        }
    }

    #[tokio::test]
    async fn screenshot_stub_env_returns_png_envelope() {
        let _guard = BRIDGE_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var(STUB_SCREENSHOT_ENV).ok();
        std::env::set_var(STUB_SCREENSHOT_ENV, "1");

        let resp = handle_request(req("screenshot", json!({ "monitor": 0 }))).await;
        let result = resp.result.expect("result");
        assert_eq!(result["format"], "png");
        assert_eq!(result["width"], 1);
        assert_eq!(result["height"], 1);
        assert_eq!(result["data"], STUB_PNG_B64);

        let ping = handle_request(req("ping", Value::Null)).await;
        let meta = ping.result.expect("ping");
        assert_eq!(meta["screenshot"], "stub");
        assert_eq!(meta["windows"], "real");

        match prev {
            Some(v) => std::env::set_var(STUB_SCREENSHOT_ENV, v),
            None => std::env::remove_var(STUB_SCREENSHOT_ENV),
        }
    }

    #[tokio::test]
    async fn ping_reports_real_screenshot_when_stub_unset() {
        let _guard = BRIDGE_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var(STUB_SCREENSHOT_ENV).ok();
        std::env::remove_var(STUB_SCREENSHOT_ENV);

        let ping = handle_request(req("ping", Value::Null)).await;
        let meta = ping.result.expect("ping");
        assert_eq!(meta["screenshot"], "real");
        assert_eq!(meta["windows"], "real");
        assert_eq!(meta["bridge"], "playcua-bridge");

        match prev {
            Some(v) => std::env::set_var(STUB_SCREENSHOT_ENV, v),
            None => std::env::remove_var(STUB_SCREENSHOT_ENV),
        }
    }

    #[tokio::test]
    async fn input_methods_ack() {
        for method in [
            "input.key",
            "input.type",
            "input.click",
            "input.scroll",
            "input.move",
        ] {
            let resp = handle_request(req(method, json!({}))).await;
            assert_eq!(resp.result.as_ref().unwrap()["ok"], true, "{method}");
        }
    }

    #[tokio::test]
    async fn windows_list_find_focus_via_native_adapters() {
        // Real enumeration may return zero or more windows; must not silently
        // invent an empty stub when the platform adapter errors.
        let list = handle_request(req("windows.list", Value::Null)).await;
        assert!(
            list.error.is_none(),
            "windows.list must succeed or fail loud: {:?}",
            list.error
        );
        let wins = list.result.expect("result");
        assert!(wins.is_array(), "windows.list must return a JSON array");

        let found = handle_request(req("windows.find", json!({ "title": "___no_such_playcua_win___" }))).await;
        assert!(found.error.is_none(), "{:?}", found.error);
        assert!(found.result.unwrap().is_null());

        // Matches host: focus is Ok on macOS/Linux honest stub; Windows may
        // succeed or fail loud for a bogus hwnd — either is acceptable.
        let focus = handle_request(req("windows.focus", json!({ "hwnd": 1 }))).await;
        if let Some(err) = focus.error {
            assert_eq!(err.code, -32603, "focus failures must be internal_error");
            assert!(err.message.contains("windows.focus"), "{}", err.message);
        } else {
            assert_eq!(focus.result.unwrap()["ok"], true);
        }
    }

    #[tokio::test]
    async fn unknown_method_fails_loud() {
        let resp = handle_request(req("process.launch", json!({}))).await;
        let err = resp.error.expect("error");
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("process.launch"));
    }

    #[tokio::test]
    async fn screenshot_invalid_params_fails_loud() {
        let resp = handle_request(req("screenshot", json!("not-an-object"))).await;
        let err = resp.error.expect("error");
        assert_eq!(err.code, -32602);
    }
}
