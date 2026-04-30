# ADR-002: Platform Adapter Selection Strategy

## Status

Accepted

## Context

bare-cua must support three primary platforms (macOS, Linux, Windows) with multiple implementation options per capability. Each platform offers different native APIs with varying capabilities and reliability characteristics.

### Platform Capability Matrix

| Capability | macOS | Linux | Windows |
|------------|-------|-------|---------|
| **Capture** | CoreGraphics, ScreenshotKit, xcap | X11, Wayland, xcap | WGC, DXGI, xcap |
| **Input** | CGEvent, NSEvent, enigo | uinput, evdev, XTest, enigo | SendInput, PostMessage, enigo |
| **Windows** | NSWorkspace, Accessibility API, xcap | EWMH, xdotool, xcap | EnumWindows, UI Automation, xcap |
| **Process** | posix_spawn, NSTask | posix_spawn, fork | CreateProcess, ShellExecute |

### Selection Criteria

1. **Reliability**: Native APIs preferred over third-party libraries
2. **Performance**: Hardware-accelerated capture over CPU-based
3. **Permissions**: Minimal permission prompts (macOS TCC, etc.)
4. **Fallback chain**: Graceful degradation when primary unavailable
5. **Maintenance burden**: Stable APIs over bleeding-edge

## Decision

We implement a **tiered adapter selection strategy** with compile-time primary selection and runtime fallback chains.

### Tier Definitions

```
┌────────────────────────────────────────────────────────────────────────────┐
│                         Adapter Selection Tiers                             │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Tier 1: Platform Native (Primary)                                        │
│  ┌────────────────┬────────────────┬────────────────────────────────┐     │
│  │    macOS       │     Linux      │           Windows              │     │
│  │  CoreGraphics  │  X11/xcap      │   Windows Graphics Capture     │     │
│  │   CGEvent      │   uinput       │        SendInput               │     │
│  │ NSWorkspace    │    EWMH        │       EnumWindows              │     │
│  └────────────────┴────────────────┴────────────────────────────────┘     │
│                              ▲                                             │
│                              │ Compile-time cfg(target_os)               │
│  ┌───────────────────────────┴─────────────────────────────────────────┐  │
│  │                     App::build() wiring                               │  │
│  │  Selects platform adapter via conditional compilation                 │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
│  Tier 2: Cross-platform Fallback (Runtime)                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │   xcap (screenshot) → available on all platforms via X11/WinAPI/Quartz │ │
│  │   enigo (input)     → pure Rust, no native dependencies             │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                              ▲                                             │
│                              │ Runtime fallback on initialization failure │
│  ┌───────────────────────────┴─────────────────────────────────────────┐  │
│  │              Platform adapter initialization                            │  │
│  │  Try primary → fallback → panic with diagnostic                       │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
│  Tier 3: Stub (Unknown Platforms)                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │   Noop adapters that return errors for unsupported platforms          │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

### Implementation Strategy

#### Compile-Time Selection

```rust
// In app/mod.rs
#[cfg(target_os = "windows")]
fn build_capture() -> Arc<dyn CapturePort> {
    use crate::adapters::windows::wgc::WgcCapture;
    use crate::adapters::xcap::XcapCapture;
    
    // Try primary (WGC), fall back to xcap
    match WgcCapture::new() {
        Ok(wgc) => {
            tracing::info!("Capture: Windows Graphics Capture (primary)");
            Arc::new(wgc)
        }
        Err(e) => {
            tracing::warn!("WGC unavailable ({}), falling back to xcap", e);
            Arc::new(XcapCapture::new())
        }
    }
}
```

#### Adapter Trait Boundaries

Each adapter implements exactly one port trait:

| Port | Windows | Linux | macOS | Fallback |
|------|---------|-------|-------|----------|
| `CapturePort` | `WgcCapture` | `X11Capture` | `CGCapture` | `XcapCapture` |
| `InputPort` | `SendInputAdapter` | `UinputAdapter` | `CGEventAdapter` | `EnigoInput` |
| `WindowPort` | `EnumWindowsAdapter` | `EwmhAdapter` | `NSWorkspaceAdapter` | Noop |
| `ProcessPort` | `NativeProcessAdapter` | `NativeProcessAdapter` | `NativeProcessAdapter` | - |
| `AnalysisPort` | `NativeAnalysisAdapter` | `NativeAnalysisAdapter` | `NativeAnalysisAdapter` | - |

### Platform-Specific Details

#### Windows

**Capture: Windows Graphics Capture (WGC)**
- **API**: `Windows.Graphics.Capture`
- **Pros**: Hardware-accelerated, window-specific capture, no permission prompts
- **Cons**: Windows 10 1903+, requires window handle for window capture
- **Fallback**: Desktop Duplication API (DXGI) → xcap

**Input: SendInput**
- **API**: `win32::UI::Input::KeyboardAndMouse::SendInput`
- **Pros**: Lowest-level input injection, works in games
- **Cons**: Requires foreground window for some applications
- **Fallback**: PostMessage → enigo

**Windows: EnumWindows**
- **API**: `win32::UI::WindowsAndMessaging::EnumWindows`
- **Pros**: Fast, comprehensive
- **Cons**: Complex window hierarchy traversal
- **Fallback**: UI Automation API

#### Linux

**Capture: X11/xcap**
- **API**: X11 `XGetImage` via x11rb + xcap
- **Pros**: Universal X11 support
- **Cons**: No Wayland support (requires PipeWire/portal)
- **Future**: PipeWire capture for Wayland

**Input: uinput**
- **API**: Linux `uinput` kernel interface
- **Pros**: Kernel-level input, works everywhere
- **Cons**: Requires udev rules or root (without capabilities)
- **Fallback**: XTest → enigo

**Windows: EWMH**
- **API**: `_NET_CLIENT_LIST` via x11rb
- **Pros**: Standard, widely supported
- **Cons**: Window managers may not implement fully
- **Fallback**: xcap window enumeration

#### macOS

**Capture: CoreGraphics**
- **API**: `CGDisplayCreateImage`, `CGWindowListCreateImage`
- **Pros**: Fast, no external dependencies
- **Cons**: Requires Screen Recording permission (TCC)
- **Future**: ScreenshotKit for modern apps

**Input: CGEvent**
- **API**: `CGEventCreateKeyboardEvent`, `CGEventPost`
- **Pros**: Official API, reliable
- **Cons**: Requires Accessibility permission (TCC)
- **Fallback**: enigo (AppleScript-based)

**Windows: NSWorkspace**
- **API**: `NSWorkspace.runningApplications`
- **Pros**: Official API, gets PID
- **Cons**: Limited window metadata
- **Complement**: `CGWindowList` for geometry

## Consequences

### Positive

1. **Graceful degradation**: Apps work even when native APIs fail
2. **Consistent interface**: Same API regardless of underlying implementation
3. **Compile-time optimization**: No runtime dispatch overhead for platform selection
4. **Testability**: Each adapter can be unit-tested in isolation
5. **Observability**: Tracing shows which adapter is active

### Negative

1. **Binary bloat**: All fallback code compiled into binary
2. **Complex initialization**: Runtime checks add startup latency
3. **Permission confusion**: Different adapters may need different permissions
4. **Feature inconsistency**: Fallbacks may lack capabilities (e.g., window-specific capture)

### Mitigations

- Feature flags to disable fallbacks for size-sensitive deployments
- Async initialization to parallelize adapter probing
- Clear documentation of permission requirements per adapter
- Capability detection API for clients to check feature availability

## Related Decisions

- ADR-001: Hexagonal Architecture with JSON-RPC 2.0 IPC
- ADR-003: Plugin System Architecture

## References

- [Windows Graphics Capture](https://docs.microsoft.com/en-us/uwp/api/windows.graphics.capture)
- [X11 Extended Window Manager Hints](https://specifications.freedesktop.org/wm-spec/wm-spec-latest.html)
- [macOS CoreGraphics](https://developer.apple.com/documentation/coregraphics)
- [Linux uinput](https://www.kernel.org/doc/html/v4.12/input/uinput.html)

## Traceability

- `@trace BCUA-ARCH-002`
- `@trace BCUA-PLATFORM-001`
