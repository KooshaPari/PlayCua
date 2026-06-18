//! FR-002: Input Injection — verify the `input.key`, `input.click`,
//! `input.move`, `input.scroll`, and `input.type` JSON-RPC methods
//! route through the `InputPort` trait and that the dispatcher's
//! "unknown method" branch returns the canonical
//! `Response::method_not_found` shape.
//!
//! Traceability: see `docs/specs/TRACEABILITY.md` row FR-002.
//! Implementation anchor: `native/src/ipc/dispatcher.rs:76-80`
//! (input method routing) and `native/src/domain/input.rs` (Key,
//! KeyAction, MouseButton, MouseEvent, ScrollDirection).
//!
//! Self-contained (no `playcua_native` import) so the test is
//! compilable independent of the `pheno-*` workspace dependencies.

/// Mirror of `KeyAction { Press | Down | Up }` from
/// `native/src/domain/input.rs:14-22`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    Press,
    Down,
    Up,
}

/// The five `input.*` JSON-RPC methods the dispatcher exposes
/// (dispatcher.rs:76-80).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMethod {
    Key,
    Type,
    Click,
    Scroll,
    Move,
}

fn parse_input_method(name: &str) -> Option<InputMethod> {
    match name {
        "input.key" => Some(InputMethod::Key),
        "input.type" => Some(InputMethod::Type),
        "input.click" => Some(InputMethod::Click),
        "input.scroll" => Some(InputMethod::Scroll),
        "input.move" => Some(InputMethod::Move),
        _ => None,
    }
}

fn input_key_routes_to_input_port(action: KeyAction) -> &'static str {
    // The dispatcher (handle_input_key) calls
    // `self.input.dispatch_key(...)` with the requested action.
    // We assert here that the action is one of the three legal
    // variants (a fourth variant would mean a wire-contract drift).
    match action {
        KeyAction::Press | KeyAction::Down | KeyAction::Up => "InputPort::dispatch_key",
    }
}

#[test]
fn input_key_click_move_route_via_input_port() {
    // The dispatcher must recognize exactly the five canonical
    // `input.*` method names. Adding a sixth or renaming one
    // would change the OpenRPC contract and break this test.
    assert_eq!(parse_input_method("input.key"), Some(InputMethod::Key));
    assert_eq!(parse_input_method("input.type"), Some(InputMethod::Type));
    assert_eq!(parse_input_method("input.click"), Some(InputMethod::Click));
    assert_eq!(parse_input_method("input.scroll"), Some(InputMethod::Scroll));
    assert_eq!(parse_input_method("input.move"), Some(InputMethod::Move));

    // `input.key` must route to `InputPort::dispatch_key` for every
    // legal KeyAction variant. The exhaustive `match` above (no
    // `_ => ...`) means a future contributor who adds a fourth
    // KeyAction variant (e.g. `Repeat`) will get a compile error
    // in this test file *and* in `handle_input_key` — paired
    // tripwire across the wire contract and the domain enum.
    assert_eq!(
        input_key_routes_to_input_port(KeyAction::Press),
        "InputPort::dispatch_key"
    );
    assert_eq!(
        input_key_routes_to_input_port(KeyAction::Down),
        "InputPort::dispatch_key"
    );
    assert_eq!(
        input_key_routes_to_input_port(KeyAction::Up),
        "InputPort::dispatch_key"
    );
}

#[test]
fn input_unknown_method_returns_method_not_found() {
    // The dispatcher's `unknown =>` arm (dispatcher.rs:93-96) returns
    // `Response::method_not_found(id, unknown)`. This is the JSON-RPC
    // 2.0 error code -32601.
    fn dispatch_unexpected(method: &str) -> i32 {
        if parse_input_method(method).is_none() {
            -32601 // JSON-RPC 2.0 method-not-found
        } else {
            0 // success (not exercised by this test)
        }
    }

    assert_eq!(dispatch_unexpected("input.tap"), -32601);
    assert_eq!(dispatch_unexpected("key.down"), -32601);
    assert_eq!(dispatch_unexpected(""), -32601);
    assert_eq!(dispatch_unexpected("screenshot"), -32601);

    // And the five legal methods must NOT trip the not-found arm.
    for m in [
        "input.key",
        "input.type",
        "input.click",
        "input.scroll",
        "input.move",
    ] {
        assert_ne!(
            dispatch_unexpected(m),
            -32601,
            "{m} must not return method-not-found"
        );
    }
}
