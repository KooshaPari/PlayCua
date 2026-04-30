# ADR-001: Hexagonal Architecture with JSON-RPC 2.0 IPC

## Status

Accepted

## Context

bare-cua requires a robust architecture for computer-use automation that must:

1. **Cross-platform compatibility**: Support macOS, Linux, and Windows with native performance
2. **Language interoperability**: Be callable from Python, C#, and other languages
3. **Process isolation**: Run as a separate process for stability and security
4. **Extensibility**: Allow third-party extensions without core modifications
5. **Type safety**: Maintain compile-time guarantees across platform abstractions

Initial analysis considered several architectural patterns:

| Pattern | Pros | Cons |
|---------|------|------|
| Monolithic FFI | Simple, fast | Tight coupling, brittle across languages |
| gRPC over TCP | Language agnostic | Requires port management, firewall issues |
| REST HTTP | Ubiquitous | Overhead, streaming challenges |
| JSON-RPC over stdio | Simple, no ports, unix philosophy | Custom protocol needed |
| Shared library | Fastest | Language lock-in, ABI complexity |

## Decision

We adopt a **hexagonal architecture** (ports and adapters) with **JSON-RPC 2.0 over stdio** as the primary IPC mechanism.

### Architecture Layers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Client (Python/C#/etc)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                              JSON-RPC 2.0 / NDJSON                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                         IPC Layer (tokio)                           │  │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐  │  │
│  │  │  Request    │───▶│ Dispatcher  │───▶│  Response (JSON)        │  │  │
│  │  │  (JSON)     │    │  (Router)   │    │  ────────────────────── │  │  │
│  │  └─────────────┘    └──────┬──────┘    └─────────────────────────┘  │  │
│  │                            │                                        │  │
│  └────────────────────────────┼────────────────────────────────────────┘  │
│                               │                                             │
│  ┌────────────────────────────┼────────────────────────────────────────┐  │
│  │                      Ports Layer (Traits)                         │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │  │
│  │  │CapturePort│ │ InputPort│ │WindowPort│ │ProcessP. │ │AnalysisP.│  │  │
│  │  │ (trait)   │ │ (trait)  │ │ (trait)  │ │ (trait)  │ │ (trait)  │  │  │
│  │  └─────┬─────┘ └─────┬────┘ └─────┬────┘ └─────┬────┘ └─────┬────┘  │  │
│  └────────┼────────────┼──────────┼────────────┼────────────┼────────┘  │
│           │            │          │            │            │           │
│  ┌────────┼────────────┼──────────┼────────────┼────────────┼────────┐  │
│  │        ▼            ▼          ▼            ▼            ▼        │  │
│  │  ┌──────────────────────────────────────────────────────────┐  │  │
│  │  │              Adapters Layer (Platform-Specific)           │  │  │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐  │  │  │
│  │  │  │   Windows   │ │    Linux    │ │       macOS         │  │  │  │
│  │  │  │  ┌───────┐  │ │  ┌───────┐  │ │  ┌───────────────┐ │  │  │  │
│  │  │  │  │  WGC  │  │ │  │  X11  │  │ │  │CoreGraphics   │ │  │  │  │
│  │  │  │  │SendInp│  │ │  │uinput │  │ │  │ CGEvent       │ │  │  │  │
│  │  │  │  │EnumWin│  │ │  │ EWMH  │  │ │  │NSWorkspace    │ │  │  │  │
│  │  │  │  └───────┘  │ │  └───────┘  │ │  └───────────────┘ │  │  │  │
│  │  │  └─────────────┘ └─────────────┘ └─────────────────────┘  │  │  │
│  │  └──────────────────────────────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Key Components

#### 1. Domain Layer (`domain/`)
Pure business types with zero external dependencies:
- `capture.rs`: `Frame`, `Monitor`, `WindowHandle`, `CaptureError`
- `input.rs`: `Key`, `KeyAction`, `MouseEvent`, `InputError`
- `window.rs`: `WindowInfo`, `WindowFilter`, `WindowError`
- `process.rs`: `ProcessHandle`, `ProcessStatus`, `ProcessError`
- `analysis.rs`: `DiffResult`, `HashResult`, `AnalysisError`

#### 2. Ports Layer (`ports/`)
Abstract trait interfaces implemented by adapters:
- `CapturePort`: Screen capture operations
- `InputPort`: Keyboard/mouse injection
- `WindowPort`: Window enumeration and focus
- `ProcessPort`: Process lifecycle management
- `AnalysisPort`: Image analysis (diff, hash)

#### 3. Adapters Layer (`adapters/`)
Platform-specific implementations:

**Windows:**
- `wgc.rs`: Windows Graphics Capture (primary)
- `sendinput.rs`: Win32 SendInput for input injection
- `enumwin.rs`: EnumWindows for window management

**Linux:**
- `x11capture.rs`: X11/xcap for capture
- `uinput.rs`: Linux uinput for input
- `ewmh.rs`: EWMH for window management

**macOS:**
- `cgcapture.rs`: CoreGraphics for capture
- `cgevent.rs`: CGEvent for input injection
- `nsworkspace.rs`: NSWorkspace for window management

**Cross-platform fallbacks:**
- `xcap.rs`: xcap library capture (fallback)
- `enigo.rs`: enigo library input (fallback)

#### 4. IPC Layer (`ipc/`)
JSON-RPC 2.0 implementation:
- `mod_types.rs`: Wire types (`Request`, `Response`, `RpcError`)
- `dispatcher.rs`: Method routing to port implementations

#### 5. App Layer (`app/`)
Dependency injection wiring selecting platform adapters at compile time.

### JSON-RPC 2.0 Protocol

**Transport:** Newline-delimited JSON (NDJSON) over stdio
- stdin: Requests from client
- stdout: Responses to client
- stderr: Structured logs (JSON)

**Message Format:**
```json
// Request
{"jsonrpc":"2.0","id":1,"method":"screenshot","params":{"monitor":0}}

// Response
{"jsonrpc":"2.0","id":1,"result":{"data":"base64...","width":1920,"height":1080,"format":"png"}}

// Error
{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"Capture failed"}}
```

## Consequences

### Positive

1. **Testability**: Domain logic is pure and easily unit-tested
2. **Platform isolation**: Platform-specific code is compartmentalized
3. **Language agnostic**: Any language that can spawn processes and parse JSON can use bare-cua
4. **No port conflicts**: stdio eliminates port binding issues
5. **Unix philosophy**: Simple, composable, pipe-friendly
6. **Type safety at boundaries**: Serde ensures protocol compliance
7. **Hot-swappable adapters**: Platform selection at compile time, runtime fallbacks

### Negative

1. **Performance overhead**: JSON parsing vs. binary protocols
2. **No streaming**: Large payloads (screenshots) must fit in memory
3. **Single connection**: stdio is 1:1, no multiplexing
4. **Error propagation**: JSON-RPC error codes require mapping from domain errors
5. **Version coupling**: Contract changes require client updates

### Mitigations

- Binary protocol could be added later as opt-in (capnp, protobuf)
- Chunked encoding for large screenshots could be implemented
- WebSocket transport could be added for remote scenarios
- OpenRPC spec provides contract versioning

## Related Decisions

- ADR-002: Platform Adapter Selection Strategy
- ADR-003: Plugin System Architecture

## References

- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [OpenRPC Specification](https://spec.open-rpc.org/)
- [The Unix Philosophy](https://en.wikipedia.org/wiki/Unix_philosophy)

## Traceability

- `@trace BCUA-ARCH-001`
- `@trace BCUA-IPC-001`
