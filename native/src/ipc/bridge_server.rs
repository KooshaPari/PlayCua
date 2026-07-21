//! Guest-side `playcua-bridge` JSON-RPC handlers (screenshot / input.* / windows.*).
//!
//! Speaks the same NDJSON JSON-RPC 2.0 surface that [`super::bridge_client::BridgeClient`]
//! expects. Screenshot returns a stub 1×1 PNG envelope today (real guest-OS
//! capture is a follow-up). Input methods ack; window list/find return empty.

use serde_json::{json, Value};
use tracing::warn;

use super::mod_types::{Request, Response};

/// Canonical 1×1 transparent PNG (base64), shared with FR-001 contract tests.
pub const STUB_PNG_B64: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

/// Dispatch one bridge request. Unknown methods → `-32601` (fail loud).
pub fn handle_request(req: Request) -> Response {
    let id = req.id.clone();
    let params = req.params.unwrap_or(Value::Null);

    match req.method.as_str() {
        "ping" => Response::ok(
            id,
            json!({
                "ok": true,
                "version": env!("CARGO_PKG_VERSION"),
                "bridge": "playcua-bridge",
                "screenshot": "stub",
            }),
        ),
        "screenshot" => handle_screenshot(id, params),
        "input.key" | "input.type" | "input.click" | "input.scroll" | "input.move" => {
            Response::ok(id, json!({ "ok": true }))
        }
        "windows.list" => Response::ok(id, json!([])),
        "windows.find" => Response::ok(id, Value::Null),
        "windows.focus" => Response::ok(id, json!({ "ok": true })),
        unknown => {
            warn!(method = %unknown, "playcua-bridge unknown method");
            Response::method_not_found(id, unknown)
        }
    }
}

fn handle_screenshot(id: Value, params: Value) -> Response {
    #[derive(serde::Deserialize, Default)]
    struct P {
        #[allow(dead_code)]
        window_title: Option<String>,
        #[allow(dead_code)]
        monitor: Option<u32>,
    }
    let _: P = match serde_json::from_value(if params.is_null() {
        json!({})
    } else {
        params
    }) {
        Ok(v) => v,
        Err(e) => return Response::invalid_params(id, e.to_string()),
    };
    Response::ok(
        id,
        json!({
            "data": STUB_PNG_B64,
            "width": 1,
            "height": 1,
            "format": "png",
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn screenshot_returns_stub_png_envelope() {
        let resp = handle_request(req("screenshot", json!({ "monitor": 0 })));
        let result = resp.result.expect("result");
        assert_eq!(result["format"], "png");
        assert_eq!(result["width"], 1);
        assert_eq!(result["height"], 1);
        assert_eq!(result["data"], STUB_PNG_B64);
    }

    #[test]
    fn input_methods_ack() {
        for method in [
            "input.key",
            "input.type",
            "input.click",
            "input.scroll",
            "input.move",
        ] {
            let resp = handle_request(req(method, json!({})));
            assert_eq!(resp.result.as_ref().unwrap()["ok"], true, "{method}");
        }
    }

    #[test]
    fn windows_list_empty_find_null_focus_ok() {
        let list = handle_request(req("windows.list", Value::Null));
        assert_eq!(list.result.unwrap(), json!([]));

        let found = handle_request(req("windows.find", json!({ "title": "x" })));
        assert!(found.result.unwrap().is_null());

        let focus = handle_request(req("windows.focus", json!({ "hwnd": 1 })));
        assert_eq!(focus.result.unwrap()["ok"], true);
    }

    #[test]
    fn unknown_method_fails_loud() {
        let resp = handle_request(req("process.launch", json!({})));
        let err = resp.error.expect("error");
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("process.launch"));
    }
}
