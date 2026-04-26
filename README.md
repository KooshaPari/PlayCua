# bare-cua

A heavy fork of [trycua/cua](https://github.com/trycua/cua) that strips the VM
layer and replaces the `computer-server` with a **native Rust binary** that
communicates via **stdio JSON-RPC 2.0**.

No Docker. No VM. No network socket. Just a subprocess pipe.

---

## Design Principles

1. **Hexagonal architecture (Ports & Adapters)** — domain types are pure Rust structs with zero external deps; ports are async traits; adapters are swappable implementations selected at compile time.
2. **SOLID** — each adapter does one thing; the dispatcher depends on abstractions not concretions; ports are the stable interfaces.
3. **KISS / DRY** — `xcap` and `enigo` are thin wrappers; platform-specific paths add only what the cross-platform fallback cannot provide (WGC, PostMessage, EnumWindows).
4. **Contract-first (OpenRPC 1.2.6)** — the full API is documented in `contracts/openrpc.json` before any code ships; clients are generated or validated against it.
5. **Polyglot / polyrepo** — Python bindings in `python/`, C# bindings in `bindings/`; each language calls the same binary over stdio.
6. **Plugin system** — additional JSON-RPC methods can be registered via `MethodPlugin` without touching the core dispatcher.
7. **Observability-first** — structured JSON logs to stderr; every adapter method is instrumented with `#[tracing::instrument]`; log level controlled by `BARE_CUA_LOG`.

---

## Architecture

```
  ┌─────────────────────────────────────────────────────────────────────┐
  │  Caller  (Python, C#, or any JSON-RPC client)                       │
  │                                                                     │
  │   Computer / NativeComputer                                         │
  │     .screenshot()  .left_click()  .type_text()  .ping()            │
  └─────────────────────────────┬───────────────────────────────────────┘
                                │  stdin/stdout  (NDJSON)
                                │  JSON-RPC 2.0
                                ▼
  ┌─────────────────────────────────────────────────────────────────────┐
  │  bare-cua-native  (Rust binary)                                     │
  │                                                                     │
  │  ┌──────────────────────────────────────────────────────────────┐   │
  │  │  main.rs  —  IPC loop (read → dispatch → write)             │   │
  │  └──────────────────────────┬───────────────────────────────────┘   │
  │                             │                                       │
  │  ┌──────────────────────────▼───────────────────────────────────┐   │
  │  │  ipc/dispatcher.rs  —  method → port call                   │   │
  │  └──┬───────────────────┬──────────────────┬────────────────────┘   │
  │     │                   │                  │                        │
  │  ┌──▼────────┐  ┌───────▼──────┐  ┌───────▼──────┐                 │
  │  │  ports/   │  │  ports/      │  │  ports/      │  (+ more)       │
  │  │  Capture  │  │  Input       │  │  Window      │                 │
  │  │  Port     │  │  Port        │  │  Port        │                 │
  │  └──┬────────┘  └───────┬──────┘  └───────┬──────┘                 │
  │     │                   │                  │                        │
  │  ┌──▼────────────────────▼──────────────────▼──────────────────┐   │
  │  │  adapters/  (selected by cfg(target_os) in app/mod.rs)      │   │
  │  │                                                             │   │
  │  │  Windows: WgcCapture, SendInputAdapter, EnumWindowsAdapter  │   │
  │  │  Linux:   X11Capture, UinputAdapter, EwmhAdapter            │   │
  │  │  macOS:   CGCapture, CGEventAdapter, NSWorkspaceAdapter     │   │
  │  │  Fallback: XcapCapture, EnigoInput                          │   │
  │  └─────────────────────────────────────────────────────────────┘   │
  └─────────────────────────────────────────────────────────────────────┘
                                │
                       native OS APIs
                                │
  ┌─────────────────────────────▼───────────────────────────────────────┐
  │  Host OS  (Windows / Linux / macOS)                                 │
  │  Running applications, game windows, etc.                           │
  └─────────────────────────────────────────────────────────────────────┘
```

### Source layout

```
native/src/
  domain/        # Pure types — Frame, Key, WindowInfo, ProcessHandle, DiffResult, …
  ports/         # Async trait interfaces — CapturePort, InputPort, WindowPort, …
  adapters/
    xcap.rs      # XcapCapture (cross-platform fallback)
    enigo.rs     # EnigoInput  (cross-platform fallback)
    windows/     # WgcCapture, SendInputAdapter, EnumWindowsAdapter
    linux/       # X11Capture, UinputAdapter, EwmhAdapter
    macos/       # CGCapture, CGEventAdapter, NSWorkspaceAdapter
    process_adapter.rs   # NativeProcessAdapter (all platforms)
    analysis_adapter.rs  # NativeAnalysisAdapter (all platforms)
  ipc/
    mod.rs        # Re-exports Request, Response, read_request, write_response
    mod_types.rs  # Wire types
    dispatcher.rs # Dispatcher — Arc<dyn Port> fields, routes methods
  app/
    mod.rs        # DI wiring: selects adapters, builds Dispatcher
  plugins/
    mod.rs        # MethodPlugin trait + PluginRegistry
  main.rs         # IPC loop — calls app::App::build(), runs forever

contracts/
  openrpc.json   # Full OpenRPC 1.2.6 spec (14 methods)

python/
  bare_cua/      # Python package
  tests/
    test_computer.py  # pytest suite with mock subprocess

native/tests/
  unit/
    analysis_tests.rs  # NativeAnalysisAdapter unit tests
```

---

## Quick Start

### 1. Build the native binary

```bash
cd native
cargo build --release
# Binary: native/target/release/bare-cua-native[.exe]
```

### 2. Install the Python package

```bash
cd python
pip install -e .
```

### 3. Use from Python

```python
import asyncio
from bare_cua import Computer

async def main():
    async with Computer("./native/target/release/bare-cua-native") as c:
        # Take a screenshot
        png_bytes = await c.screenshot()
        with open("screen.png", "wb") as f:
            f.write(png_bytes)

        # Click, type, scroll
        await c.left_click(960, 540)
        await c.type_text("hello world")
        await c.scroll(960, 540, direction="down", amount=5)

        # Window management
        windows = await c.list_windows()
        print(windows)

        # Wait for a visual change
        changed_frame = await c.wait_for_visual_change(timeout=10.0)

asyncio.run(main())
```

### 4. Use the agent

```python
import asyncio
from bare_cua import Computer, ComputerAgent

async def main():
    async with Computer() as computer:
        agent = ComputerAgent(computer, model="claude-sonnet-4-5")
        result = await agent.run("Open Notepad and type 'Hello from bare-cua'")
        print(result)

asyncio.run(main())
```

### 5. Use from C# (DINOForge integration)

```csharp
using BareCua;

await using var computer = await NativeComputer.StartAsync("bare-cua-native");

byte[] png = await computer.ScreenshotAsync(windowTitle: "Diplomacy is Not an Option");
await computer.ClickAsync(960, 540);
await computer.TypeTextAsync("hello");
await computer.PressKeyAsync("return");

var windows = await computer.ListWindowsAsync();
```

---

## JSON-RPC Protocol

The native binary reads newline-delimited JSON requests from **stdin** and
writes newline-delimited JSON responses to **stdout**. Logs go to **stderr**.

The full API spec is in [`contracts/openrpc.json`](contracts/openrpc.json).

### Request format
```json
{"jsonrpc":"2.0","id":1,"method":"screenshot","params":{"monitor":0}}
```

### Response format (success)
```json
{"jsonrpc":"2.0","id":1,"result":{"data":"<base64>","width":1920,"height":1080,"format":"png"}}
```

### Response format (error)
```json
{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found: foo"}}
```

### Available methods

| Method            | Params                                                          | Result                                      |
|-------------------|-----------------------------------------------------------------|---------------------------------------------|
| `ping`            | `{}`                                                            | `{"ok":true,"version":"0.1.0"}`             |
| `screenshot`      | `{"window_title?":str, "monitor?":int}`                        | `{"data":b64,"width":int,"height":int,...}` |
| `input.key`       | `{"key":str, "action":"press"\|"down"\|"up"}`                    | `{"ok":true}`                               |
| `input.type`      | `{"text":str}`                                                  | `{"ok":true}`                               |
| `input.click`     | `{"x":int,"y":int,"button":"left"\|"right"\|"middle","action":"click"\|"down"\|"up"}` | `{"ok":true}` |
| `input.scroll`    | `{"x":int,"y":int,"direction":"up"\|"down"\|"left"\|"right","amount?":int}` | `{"ok":true}` |
| `input.move`      | `{"x":int,"y":int}`                                            | `{"ok":true}`                               |
| `windows.list`    | `{}`                                                            | `[{hwnd,title,pid,x,y,width,height,visible}]` |
| `windows.focus`   | `{"hwnd":int}`                                                  | `{"ok":true}`                               |
| `windows.find`    | `{"title?":str,"pid?":int}`                                    | `{...WindowInfo...}` or `null`              |
| `process.launch`  | `{"path":str,"args?":[],"cwd?":str}`                           | `{"pid":int}`                               |
| `process.kill`    | `{"pid":int}`                                                   | `{"ok":true}`                               |
| `process.status`  | `{"pid":int}`                                                   | `{"running":bool,"exit_code?":int}`         |
| `analysis.diff`   | `{"image_a":b64,"image_b":b64,"threshold?":float}`             | `{"changed":bool,"change_ratio":float}`     |
| `analysis.hash`   | `{"image":b64}`                                                 | `{"hash":str}`                              |

---

## Platform Support

| Feature              | Windows             | Linux               | macOS               |
|----------------------|---------------------|---------------------|---------------------|
| Monitor screenshot   | xcap (fallback)     | xcap/x11rb          | xcap/CG             |
| Window screenshot    | WGC (GPU) + xcap    | xcap/x11rb          | xcap/CG             |
| Keyboard input       | enigo (SendInput)   | enigo (uinput/X11)  | enigo (CGEvent)     |
| Mouse input          | enigo               | enigo               | enigo               |
| PostMessage inject   | Win32 PostMessage   | N/A                 | N/A                 |
| Window list          | Win32 EnumWindows   | xcap/EWMH           | xcap/NSWorkspace    |
| Window focus         | SetForegroundWindow | TODO (x11rb)        | TODO (NSWorkspace)  |
| Process launch/kill  | Win32 Terminate     | SIGTERM             | SIGTERM             |
| Structured logs      | JSON to stderr      | JSON to stderr      | JSON to stderr      |

---

## Environment Variables

| Variable       | Default  | Description                                            |
|----------------|----------|--------------------------------------------------------|
| `BARE_CUA_LOG` | `info`   | Tracing level: `error`, `warn`, `info`, `debug`, `trace` |

---

## Extending with Plugins

Register custom JSON-RPC methods without touching the core:

```rust
use bare_cua_native::plugins::{MethodPlugin, PluginRegistry};
use async_trait::async_trait;
use serde_json::Value;

struct MyPlugin;

#[async_trait]
impl MethodPlugin for MyPlugin {
    fn method_name(&self) -> &'static str { "custom.hello" }
    async fn handle(&self, _params: Value) -> anyhow::Result<Value> {
        Ok(serde_json::json!({ "greeting": "hello from plugin" }))
    }
}

let mut registry = PluginRegistry::new();
registry.register(Box::new(MyPlugin));
```

---

## Comparison with upstream CUA

| Feature             | upstream CUA             | bare-cua                        |
|---------------------|--------------------------|---------------------------------|
| Computer server     | Python + macOS VM        | Rust binary (cross-platform)    |
| Transport           | Unix socket / HTTP       | stdio NDJSON (no server needed) |
| VM dependency       | Required                 | None                            |
| Docker              | Required                 | None                            |
| Screenshot (games)  | macOS only               | Windows WGC (GPU-accelerated)   |
| Input injection     | macOS only               | enigo + Win32 PostMessage       |
| Architecture        | Monolithic               | Hexagonal (ports & adapters)    |
| Languages           | Python                   | Rust + Python + C# bindings     |
| Startup overhead    | ~5s (VM boot)            | <100ms (process spawn)          |
| Observability       | Basic logs               | JSON structured tracing         |

---

## License

MIT — see [LICENSE](./LICENSE).
