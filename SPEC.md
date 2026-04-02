# bare-cua Specification

> Native computer automation via JSON-RPC over stdio

## Overview

bare-cua is a native Rust binary that provides computer automation capabilities (screenshot, input, window management) via JSON-RPC 2.0 over stdio. No Docker, no VM, no network socket - just a subprocess pipe.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         bare-cua Architecture                                  │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        Client Layer (Python, C#, etc.)                  │   │
│   │                                                                       │   │
│   │   Computer / NativeComputer                                           │   │
│   │     .screenshot()  .left_click()  .type_text()  .ping()               │   │
│   └─────────────────────────────┬───────────────────────────────────────┘   │
│                                   │  stdin/stdout (NDJSON)                    │
│                                   │  JSON-RPC 2.0                             │
│                                   ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     bare-cua-native (Rust binary)                      │   │
│   │                                                                       │   │
│   │  ┌──────────────────────────────────────────────────────────────┐    │   │
│   │  │  main.rs  —  IPC loop (read → dispatch → write)              │    │   │
│   │  └──────────────────────────┬───────────────────────────────────┘    │   │
│   │                            │                                          │   │
│   │  ┌─────────────────────────▼──────────────────────────────────────┐   │   │
│   │  │  ipc/dispatcher.rs  —  method → port call                      │   │   │
│   │  └──┬───────────────────┬──────────────────┬─────────────────────┘   │   │
│   │     │                   │                  │                         │   │
│   │  ┌──▼────────┐  ┌───────▼──────┐  ┌───────▼──────┐                 │   │
│   │  │  ports/   │  │  ports/      │  │  ports/      │  (+ more)        │   │
│   │  │  Capture  │  │  Input       │  │  Window      │                  │   │
│   │  │  Port     │  │  Port        │  │  Port        │                  │   │
│   │  └──┬────────┘  └───────┬──────┘  └───────┬──────┘                  │   │
│   │     │                   │                  │                         │   │
│   │  ┌──▼────────┐  ┌───────▼──────┐  ┌───────▼──────┐                 │   │
│   │  │adapters/  │  │adapters/     │  │adapters/     │                 │   │
│   │  │  xcap    │  │  enigo       │  │  sysinfo     │                 │   │
│   │  │  (wgc)   │  │  (rdev)      │  │  (wmctrl)    │                 │   │
│   │  └──────────┘  └──────────────┘  └──────────────┘                 │   │
│   │                                                                       │   │
│   └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Design Principles

1. **Hexagonal Architecture**: Domain types are pure Rust structs with zero external deps; ports are async traits; adapters are swappable
2. **SOLID**: Each adapter does one thing; dispatcher depends on abstractions
3. **Contract-first (OpenRPC 1.2.6)**: Full API documented in `contracts/openrpc.json` before code
4. **Polyglot**: Python bindings in `python/`, C# bindings in `bindings/`
5. **Plugin System**: Additional JSON-RPC methods via `MethodPlugin`
6. **Observability-first**: Structured JSON logs; `#[tracing::instrument]` on all methods

## JSON-RPC API

### Methods

| Method | Description | Parameters | Returns |
|--------|-------------|------------|---------|
| `ping` | Health check | - | `{ "pong": true }` |
| `screenshot` | Capture screen | `display_id?` | `{ "data": base64_png }` |
| `left_click` | Mouse left click | `x, y` | `{ "success": true }` |
| `right_click` | Mouse right click | `x, y` | `{ "success": true }` |
| `type_text` | Type text | `text` | `{ "success": true }` |
| `press_key` | Press key | `key` | `{ "success": true }` |
| `get_window_list` | List windows | - | `{ "windows": [...] }` |
| `focus_window` | Focus window | `window_id` | `{ "success": true }` |
| `resize_window` | Resize window | `window_id, width, height` | `{ "success": true }` |
| `get_cursor_position` | Get mouse pos | - | `{ "x": i32, "y": i32 }` |

### Example Request/Response

```json
// Request
{"jsonrpc": "2.0", "id": 1, "method": "screenshot", "params": {"display_id": 0}}

// Response
{"jsonrpc": "2.0", "id": 1, "result": {"data": "iVBORw0KGgo...", "width": 1920, "height": 1080}}
```

## Ports (Traits)

```rust
#[async_trait]
pub trait CapturePort: Send + Sync {
    async fn screenshot(&self, display_id: u32) -> Result<Screenshot>;
    async fn list_displays(&self) -> Result<Vec<DisplayInfo>>;
}

#[async_trait]
pub trait InputPort: Send + Sync {
    async fn move_mouse(&self, x: i32, y: i32) -> Result<()>;
    async fn left_click(&self, x: i32, y: i32) -> Result<()>;
    async fn type_text(&self, text: &str) -> Result<()>;
    async fn press_key(&self, key: &str) -> Result<()>;
}

#[async_trait]
pub trait WindowPort: Send + Sync {
    async fn list_windows(&self) -> Result<Vec<WindowInfo>>;
    async fn focus_window(&self, id: u64) -> Result<()>;
    async fn resize_window(&self, id: u64, width: u32, height: u32) -> Result<()>;
}
```

## Adapters

| Port | Adapter | Platform | Implementation |
|------|---------|----------|----------------|
| Capture | xcap | macOS, Linux | Native platform APIs |
| Capture | wgc | Windows | Windows.Graphics.Capture |
| Input | enigo | All | Cross-platform input |
| Input | rdev | All | Alternative input |
| Window | sysinfo | All | Process info |
| Window | wmctrl | Linux | X11 window control |
| Window | PostMessage | Windows | Win32 messages |

## Performance Targets

| Metric | Target |
|--------|--------|
| Screenshot latency | <50ms |
| Input latency | <10ms |
| Window operation latency | <20ms |
| JSON-RPC overhead | <1ms |
| Memory usage | <50MB |
| Startup time | <100ms |

## Security

- No network exposure - stdio only
- No subprocess shell execution
- Sandboxed to user permissions
- Audit logging of all actions

## References

- [JSON-RPC 2.0](https://www.jsonrpc.org/specification)
- [OpenRPC](https://open-rpc.org/)
- [xcap](https://github.com/nashaofu/xcap)
- [enigo](https://github.com/enigo-rs/enigo)
