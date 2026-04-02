# bare-cua Plan

## Overview
Native Rust binary replacing VM layer with stdio JSON-RPC 2.0.

## Phases

### Phase 1: Core IPC (2 weeks)
- JSON-RPC dispatcher implementation
- Request/response handling
- Plugin system

### Phase 2: Platform Adapters (3 weeks)
- Windows: WGC, SendInput, EnumWindows
- Linux: X11, Uinput, EWMH
- macOS: CG, CGEvent, NSWorkspace

### Phase 3: Cross-Language (1 week)
- Python bindings
- C# bindings
- OpenRPC spec validation

## Deliverables
- Native binary with JSON-RPC over stdio
- Platform-specific adapters (Windows/Linux/macOS)
- Python and C# client libraries
- OpenRPC contract documentation