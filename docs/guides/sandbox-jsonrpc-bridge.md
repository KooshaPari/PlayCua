# Sandbox JSON-RPC bridge (capture / input / window)

After M2 process dispatch (#137), sandbox modality routes
**capture / input / window** through a stdio NDJSON JSON-RPC tunnel to a
guest-side `playcua-bridge` binary — never through native host adapters.

## Driver-managed spawn

[`SandboxDriver`](../../native/src/modality/sandbox.rs) owns the bridge
child alongside the sandbox guest:

1. Guest spawn (`WireSandboxAdapter::spawn`) calls
   `SandboxDriver::spawn_guest` then `ensure_bridge` →
   `SandboxDriver::spawn_bridge`.
2. Bridge binary: `PLAYCUA_BRIDGE_BIN` if set, else `playcua-bridge` on
   `$PATH`. Missing → **fail loud** (no native host leak).
3. I/O ports (`SandboxBridgePorts::from_shared_slot`) reuse the
   driver-published `BridgeClient` instead of a separate ambient PATH spawn.

### Direct backend (hermetic)

```bash
export PLAYCUA_SANDBOX_BACKEND=direct
export PLAYCUA_BRIDGE_BIN="$PWD/native/tests/fixtures/fake-playcua-bridge.sh"
```

`SandboxDriver::spawn_bridge` / `spawn_guest_with_bridge` exec the fake
script as a live stdio child so CI exercises the real spawn path without
a production bridge or firejail/sandbox-exec on `$PATH`.

## Protocol sketch

Newline-delimited JSON-RPC 2.0 (same method names as `playcua-native`):

```text
host → bridge:  {"jsonrpc":"2.0","id":1,"method":"screenshot","params":{"monitor":0}}\n
bridge → host:  {"jsonrpc":"2.0","id":1,"result":{"data":"<b64>","width":W,"height":H,"format":"png"}}\n
```

| Port method | RPC method | Notes |
|-------------|------------|--------|
| `capture_display` / `capture_window` | `screenshot` | `monitor` or `window_title` |
| `key_event` / `type_text` / `mouse_event` | `input.key` / `input.type` / `input.click` / `input.scroll` / `input.move` | same shapes as public IPC |
| `list_windows` / `find_window` / `focus_window` | `windows.list` / `windows.find` / `windows.focus` | |

## Configuration

| Env | Purpose |
|-----|---------|
| `PLAYCUA_BRIDGE_BIN` | Absolute path to bridge binary (or hermetic fake) |
| `PLAYCUA_SANDBOX_BACKEND` | Sandbox wrapper for `process.launch` (`direct` for CI) |

If the bridge binary is missing, I/O ports **and** guest spawn **fail loud**
with an actionable error. There is no silent fallback to native
capture/input/window.

## Hermetic local check

```bash
# Duplex + unit coverage
cargo test --locked --manifest-path native/Cargo.toml bridge

# Spawn the fixture script as PLAYCUA_BRIDGE_BIN via SandboxDriver
cargo test --locked --manifest-path native/Cargo.toml \
  --test hermetic_sandbox_bridge

# Driver guest + bridge alongside
cargo test --locked --manifest-path native/Cargo.toml \
  --test hermetic_sandbox_driver
```

Fixture: `native/tests/fixtures/fake-playcua-bridge.{sh,py}`.
