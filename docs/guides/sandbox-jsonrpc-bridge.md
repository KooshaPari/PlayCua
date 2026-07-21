# Sandbox JSON-RPC bridge (capture / input / window)

After M2 process dispatch (#137), sandbox modality routes
**capture / input / window** through a stdio NDJSON JSON-RPC tunnel to a
guest-side `playcua-bridge` binary — never through native host adapters.

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

If the bridge binary is missing, I/O ports **fail loud** with an actionable
error. There is no silent fallback to native capture/input/window.

## Hermetic local check

```bash
# Duplex + unit coverage
cargo test --locked --manifest-path native/Cargo.toml bridge

# Spawn the fixture script as PLAYCUA_BRIDGE_BIN
cargo test --locked --manifest-path native/Cargo.toml \
  --test hermetic_sandbox_bridge
```

Fixture: `native/tests/fixtures/fake-playcua-bridge.{sh,py}`.
