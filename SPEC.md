# bare-cua Specification

**Version**: 0.1.0  
**Status**: Draft  
**Last Updated**: 2026-04-04  
**Authors**: bare-cua contributors  

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Architecture Overview](#2-architecture-overview)
3. [Domain Layer](#3-domain-layer)
4. [Ports Layer](#4-ports-layer)
5. [Adapters Layer](#5-adapters-layer)
6. [IPC Layer](#6-ipc-layer)
7. [Application Layer](#7-application-layer)
8. [Plugin System](#8-plugin-system)
9. [API Reference](#9-api-reference)
10. [Platform-Specific Details](#10-platform-specific-details)
11. [Performance Characteristics](#11-performance-characteristics)
12. [Security Model](#12-security-model)
13. [Testing Strategy](#13-testing-strategy)
14. [Deployment](#14-deployment)
15. [Extensibility](#15-extensibility)
16. [Future Work](#16-future-work)
17. [Appendices](#17-appendices)

---

## 1. Introduction

### 1.1 Purpose

bare-cua is a cross-platform Computer-Use Agent (CUA) framework that provides native automation capabilities through a JSON-RPC 2.0 interface. It enables AI agents and automation scripts to interact with desktop environments programmatically.

### 1.2 Scope

The specification covers:
- Core architecture (hexagonal/ports-and-adapters)
- Domain types and data models
- Platform abstraction interfaces (ports)
- Platform-specific implementations (adapters)
- JSON-RPC 2.0 wire protocol
- Plugin extension system
- Security and permission models
- Performance characteristics
- Testing and validation

### 1.3 Design Principles

| Principle | Description | Implementation |
|-----------|-------------|----------------|
| **Zero-cost abstractions** | No runtime overhead for type safety | Rust traits, compile-time dispatch |
| **Cross-platform** | Native performance on macOS, Linux, Windows | Conditional compilation, adapter selection |
| **Language agnostic** | Usable from any language with process spawning | JSON-RPC over stdio |
| **Extensible** | Third-party extensions without core changes | Plugin trait system |
| **Secure** | Minimal permissions, user consent | TCC integration, capability-based security |
| **Observable** | Full visibility into operation | Structured logging, tracing |
| **Testable** | Comprehensive test coverage | Pure domain logic, mock adapters |

### 1.4 Terminology

| Term | Definition |
|------|------------|
| **CUA** | Computer-Use Agent - software that automates desktop interactions |
| **Domain** | Pure business logic with no external dependencies |
| **Port** | Abstract interface (trait) defining a capability |
| **Adapter** | Concrete implementation of a port for a specific platform |
| **IPC** | Inter-Process Communication - how clients talk to bare-cua |
| **NDJSON** | Newline-delimited JSON - streaming format for JSON-RPC |

### 1.5 References

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [OpenRPC Specification](https://spec.open-rpc.org/)
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- See `docs/adr/` for Architecture Decision Records
- See `docs/research/SOTA-2026.md` for academic and industry context

---

## 2. Architecture Overview

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           bare-cua Architecture                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │                           Client Layer                                     ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       ││
│  │  │   Python    │  │    C#       │  │   Node.js   │  │     Go      │       ││
│  │  │   Client    │  │   Client    │  │   Client    │  │   Client    │       ││
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       ││
│  │         │                │                │                │              ││
│  │         └────────────────┴────────────────┴────────────────┘              ││
│  │                              │                                            ││
│  │                    JSON-RPC 2.0 over stdio                               ││
│  │                    (newline-delimited)                                   ││
│  └──────────────────────────────┬────────────────────────────────────────────┘│
│                                 │                                               │
│  ┌──────────────────────────────┼────────────────────────────────────────────┐│
│  │                              ▼              IPC Layer                       ││
│  │  ┌──────────────────────────────────────────────────────────────────────┐  ││
│  │  │                         JSON-RPC Dispatcher                         │  ││
│  │  │  - Request parsing (serde)                                         │  ││
│  │  │  - Method routing                                                  │  ││
│  │  │  - Response serialization                                          │  ││
│  │  │  - Error handling (JSON-RPC codes)                                 │  ││
│  │  └───────────────────────────┬───────────────────────────────────────┘  ││
│  │                              │                                            ││
│  │  ┌───────────────────────────┴────────────────────────────────────────┐  ││
│  │  │                         Plugin Registry                             │  ││
│  │  │  - Compile-time plugin registration                                 │  ││
│  │  │  - Method name resolution                                           │  ││
│  │  └───────────────────────────┬───────────────────────────────────────┘  ││
│  └──────────────────────────────┼────────────────────────────────────────────┘│
│                                 │                                               │
│  ┌──────────────────────────────┼────────────────────────────────────────────┐│
│  │                              ▼              Ports Layer (Traits)           ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   ││
│  │  │ Capture  │  │  Input   │  │  Window  │  │ Process  │  │ Analysis │   ││
│  │  │   Port   │  │   Port   │  │   Port   │  │   Port   │  │   Port   │   ││
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   ││
│  │       │             │             │             │             │         ││
│  │       └─────────────┴─────────────┴─────────────┴─────────────┘         ││
│  │                     Platform-agnostic interfaces                         ││
│  └──────────────────────────────────────────────────────────────────────────┘│
│                                                                                 │
│  ┌────────────────────────────────────────────────────────────────────────────┐│
│  │                         Adapters Layer (Implementations)                   ││
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐           ││
│  │  │     macOS       │  │     Linux       │  │    Windows      │           ││
│  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │           ││
│  │  │  │CoreGraphics│  │  │  │    X11    │  │  │    WGC    │  │           ││
│  │  │  │  CGEvent   │  │  │  │  uinput   │  │  │ SendInput │  │           ││
│  │  │  │NSWorkspace │  │  │  │   EWMH    │  │  │ EnumWindows│ │           ││
│  │  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │           ││
│  │  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │           ││
│  │  │  │   xcap   │  │  │  │   xcap   │  │  │  │   xcap   │  │           ││
│  │  │  │  enigo   │  │  │  │  enigo   │  │  │  │  enigo   │  │           ││
│  │  │  │ (fallback)│  │  │  │ (fallback)│  │  │  │ (fallback)│  │           ││
│  │  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │           ││
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘           ││
│  │                    Platform-specific + cross-platform fallbacks            ││
│  └──────────────────────────────────────────────────────────────────────────┘│
│                                                                                 │
│  ┌────────────────────────────────────────────────────────────────────────────┐│
│  │                           Domain Layer (Pure Types)                        ││
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌─────────────┐ ││
│  │  │    Frame      │  │    Key        │  │  WindowInfo   │  │ProcessHandle│ ││
│  │  │  WindowHandle │  │  MouseEvent   │  │ WindowFilter  │  │ProcessStatus│ ││
│  │  │   Monitor     │  │   KeyAction   │  │  WindowError  │  │ ProcessError│ ││
│  │  └───────────────┘  └───────────────┘  └───────────────┘  └─────────────┘ ││
│  │                    Zero external dependencies                              ││
│  └──────────────────────────────────────────────────────────────────────────┘│
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Layer Responsibilities

| Layer | Responsibility | Dependencies |
|-------|----------------|--------------|
| **Domain** | Pure business types | `std`, `serde` |
| **Ports** | Abstract capability interfaces | `domain`, `async-trait` |
| **Adapters** | Platform-specific implementations | `ports`, platform crates |
| **IPC** | JSON-RPC wire protocol | `serde_json`, `tokio` |
| **App** | Dependency injection wiring | All above |
| **Plugins** | Extension system | `ipc`, `anyhow` |

### 2.3 Dependency Flow

```
Dependencies flow inward (outer layers depend on inner):

┌─────────────────────────────────────────────────────────────┐
│                    Adapters Layer                          │
│                         │                                   │
│                         ▼                                   │
│                    Ports Layer                             │
│                         │                                   │
│                         ▼                                   │
│                    Domain Layer                              │
│                    (no dependencies outward)                │
└─────────────────────────────────────────────────────────────┘
```

### 2.4 Compile-Time vs Runtime

| Decision | Type | Mechanism |
|----------|------|-----------|
| Platform selection | Compile-time | `#[cfg(target_os = ...)]` |
| Adapter fallbacks | Runtime | `match` on initialization result |
| Plugin registration | Compile-time | Trait object registration |
| Method dispatch | Runtime | `match` on method name |

---

## 3. Domain Layer

### 3.1 Design Principles

The domain layer contains pure business types with:
- **Zero external dependencies** (only `std` and `serde`)
- **No platform-specific code**
- **Immutable data structures**
- **Strong typing** (newtypes over primitives)

### 3.2 Capture Domain

#### 3.2.1 Frame

```rust
/// A captured frame: raw PNG bytes plus dimensions.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Base64-encoded PNG bytes (matches IPC contract).
    pub data: String,
    pub width: u32,
    pub height: u32,
}

impl Frame {
    /// Create a new frame with validated dimensions.
    pub fn new(data: String, width: u32, height: u32) -> Self {
        Self { data, width, height }
    }

    /// Calculate approximate size in bytes.
    pub fn approximate_size_bytes(&self) -> usize {
        self.data.len() * 3 / 4 // base64 decode estimate
    }

    /// Check if dimensions are reasonable (sanity check).
    pub fn is_valid(&self) -> bool {
        self.width > 0 
            && self.height > 0 
            && self.width <= 16384 
            && self.height <= 16384
    }
}
```

#### 3.2.2 Monitor

```rust
/// Identifies a physical monitor by index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Monitor(pub u32);

impl Monitor {
    pub const PRIMARY: Self = Self(0);

    pub fn index(&self) -> u32 {
        self.0
    }
}

impl Default for Monitor {
    fn default() -> Self {
        Self::PRIMARY
    }
}
```

#### 3.2.3 WindowHandle

```rust
/// Opaque handle to an OS window.
/// - Windows: HWND (usize)
/// - Linux: X11 Window ID (u64)
/// - macOS: CGWindowID (u32)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowHandle(pub usize);

impl WindowHandle {
    pub fn as_usize(&self) -> usize {
        self.0
    }

    /// Platform-specific conversion to native handle.
    #[cfg(target_os = "windows")]
    pub fn as_hwnd(&self) -> windows::Win32::Foundation::HWND {
        windows::Win32::Foundation::HWND(self.0 as *mut _)
    }

    #[cfg(target_os = "linux")]
    pub fn as_xid(&self) -> u64 {
        self.0 as u64
    }

    #[cfg(target_os = "macos")]
    pub fn as_cgwindowid(&self) -> u32 {
        self.0 as u32
    }
}
```

#### 3.2.4 CaptureError

```rust
/// Errors that can arise during screen capture.
#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("window not found: {0}")]
    WindowNotFound(String),
    
    #[error("monitor not found: index={0}")]
    MonitorNotFound(u32),
    
    #[error("capture failed: {0}")]
    CaptureFailed(String),
    
    #[error("encode failed: {0}")]
    EncodeFailed(String),
    
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("timeout after {0:?}")]
    Timeout(std::time::Duration),
}

impl CaptureError {
    /// Map to JSON-RPC error code.
    pub fn to_error_code(&self) -> i32 {
        match self {
            Self::WindowNotFound(_) => -32001,
            Self::MonitorNotFound(_) => -32002,
            Self::CaptureFailed(_) => -32003,
            Self::EncodeFailed(_) => -32004,
            Self::PermissionDenied(_) => -32005,
            Self::Timeout(_) => -32006,
        }
    }
}
```

### 3.3 Input Domain

#### 3.3.1 Key

```rust
/// A keyboard key identifier (string-based for cross-platform portability).
/// Common key names: "a", "return", "escape", "f1", "ctrl", "shift", "space", "tab"
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key(pub String);

impl Key {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into().to_lowercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a modifier key.
    pub fn is_modifier(&self) -> bool {
        matches!(self.0.as_str(), "ctrl" | "alt" | "shift" | "meta" | "command" | "win")
    }

    /// Check if this is a navigation key.
    pub fn is_navigation(&self) -> bool {
        matches!(self.0.as_str(), "up" | "down" | "left" | "right" | "home" | "end" | "pageup" | "pagedown")
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Key {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}
```

#### 3.3.2 KeyAction

```rust
/// The direction/lifecycle of a key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Full press-and-release cycle.
    Press,
    /// Key-down only.
    Down,
    /// Key-up only.
    Up,
}

impl KeyAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Press => "press",
            Self::Down => "down",
            Self::Up => "up",
        }
    }
}

impl Default for KeyAction {
    fn default() -> Self {
        Self::Press
    }
}
```

#### 3.3.3 MouseButton

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Middle => "middle",
        }
    }
}

impl Default for MouseButton {
    fn default() -> Self {
        Self::Left
    }
}
```

#### 3.3.4 MouseAction

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    /// Full click cycle (down + up).
    Click,
    /// Button-down only.
    Down,
    /// Button-up only.
    Up,
}

impl MouseAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::Down => "down",
            Self::Up => "up",
        }
    }
}
```

#### 3.3.5 ScrollDirection

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// Get axis and sign for input injection.
    pub fn to_delta(&self, amount: i32) -> (i32, i32) {
        match self {
            Self::Up => (0, -amount),
            Self::Down => (0, amount),
            Self::Left => (-amount, 0),
            Self::Right => (amount, 0),
        }
    }
}
```

#### 3.3.6 MouseEvent

```rust
#[derive(Debug, Clone)]
pub enum MouseEvent {
    /// Move cursor to absolute position.
    Move { x: i32, y: i32 },
    /// Click, press, or release at position.
    Click {
        x: i32,
        y: i32,
        button: MouseButton,
        action: MouseAction,
    },
    /// Scroll at position.
    Scroll {
        x: i32,
        y: i32,
        direction: ScrollDirection,
        amount: i32,
    },
}

impl MouseEvent {
    /// Get the position for this event.
    pub fn position(&self) -> (i32, i32) {
        match self {
            Self::Move { x, y } => (*x, *y),
            Self::Click { x, y, .. } => (*x, *y),
            Self::Scroll { x, y, .. } => (*x, *y),
        }
    }

    /// Create a simple click event.
    pub fn click(x: i32, y: i32) -> Self {
        Self::Click {
            x,
            y,
            button: MouseButton::Left,
            action: MouseAction::Click,
        }
    }
}
```

#### 3.3.7 InputError

```rust
#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("unknown key: {0}")]
    UnknownKey(String),
    
    #[error("injection failed: {0}")]
    InjectionFailed(String),
    
    #[error("device initialization failed: {0}")]
    InitFailed(String),
    
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("invalid coordinates: ({0}, {1})")]
    InvalidCoordinates(i32, i32),
}

impl InputError {
    pub fn to_error_code(&self) -> i32 {
        match self {
            Self::UnknownKey(_) => -32101,
            Self::InjectionFailed(_) => -32102,
            Self::InitFailed(_) => -32103,
            Self::PermissionDenied(_) => -32104,
            Self::InvalidCoordinates(_, _) => -32105,
        }
    }
}
```

### 3.4 Window Domain

#### 3.4.1 WindowInfo

```rust
/// Metadata about a top-level OS window.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowInfo {
    /// Platform window handle (HWND on Windows, XID on Linux, etc.).
    pub hwnd: usize,
    pub title: String,
    pub pid: u32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub visible: bool,
}

impl WindowInfo {
    /// Create a new WindowInfo with validation.
    pub fn new(
        hwnd: usize,
        title: impl Into<String>,
        pid: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        visible: bool,
    ) -> Self {
        Self {
            hwnd,
            title: title.into(),
            pid,
            x,
            y,
            width,
            height,
            visible,
        }
    }

    /// Get the window handle.
    pub fn handle(&self) -> WindowHandle {
        WindowHandle(self.hwnd)
    }

    /// Get the window rectangle.
    pub fn rect(&self) -> (i32, i32, i32, i32) {
        (self.x, self.y, self.width, self.height)
    }

    /// Check if a point is inside this window.
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x 
            && px < self.x + self.width 
            && py >= self.y 
            && py < self.y + self.height
    }

    /// Get the center point of the window.
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
}
```

#### 3.4.2 WindowFilter

```rust
/// Filter criteria for finding a specific window.
#[derive(Debug, Default, Clone)]
pub struct WindowFilter {
    /// Case-insensitive substring match against window title.
    pub title: Option<String>,
    /// Exact match on process ID.
    pub pid: Option<u32>,
    /// Match visible windows only.
    pub visible_only: bool,
}

impl WindowFilter {
    pub fn by_title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            pid: None,
            visible_only: true,
        }
    }

    pub fn by_pid(pid: u32) -> Self {
        Self {
            title: None,
            pid: Some(pid),
            visible_only: true,
        }
    }

    /// Check if a window matches this filter.
    pub fn matches(&self, window: &WindowInfo) -> bool {
        if self.visible_only && !window.visible {
            return false;
        }

        if let Some(ref title) = self.title {
            if !window.title.to_lowercase().contains(&title.to_lowercase()) {
                return false;
            }
        }

        if let Some(pid) = self.pid {
            if window.pid != pid {
                return false;
            }
        }

        true
    }
}
```

#### 3.4.3 WindowError

```rust
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("window not found")]
    NotFound,
    
    #[error("window not found with filter: {0:?}")]
    NotFoundWithFilter(WindowFilter),
    
    #[error("enumeration failed: {0}")]
    EnumerationFailed(String),
    
    #[error("operation failed: {0}")]
    Failed(String),
    
    #[error("permission denied: {0}")]
    PermissionDenied(String),
}

impl WindowError {
    pub fn to_error_code(&self) -> i32 {
        match self {
            Self::NotFound => -32201,
            Self::NotFoundWithFilter(_) => -32202,
            Self::EnumerationFailed(_) => -32203,
            Self::Failed(_) => -32204,
            Self::PermissionDenied(_) => -32205,
        }
    }
}
```

### 3.5 Process Domain

#### 3.5.1 ProcessHandle

```rust
/// A request to launch a process.
#[derive(Debug, Clone)]
pub struct ProcessHandle {
    pub path: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

impl ProcessHandle {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            args: Vec::new(),
            cwd: None,
            env: None,
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Build the command string for logging.
    pub fn to_command_string(&self) -> String {
        let mut cmd = self.path.clone();
        for arg in &self.args {
            cmd.push(' ');
            if arg.contains(' ') {
                cmd.push('"');
                cmd.push_str(arg);
                cmd.push('"');
            } else {
                cmd.push_str(arg);
            }
        }
        cmd
    }
}
```

#### 3.5.2 ProcessStatus

```rust
/// The runtime status of a managed process.
#[derive(Debug, Clone)]
pub struct ProcessStatus {
    pub running: bool,
    pub exit_code: Option<i32>,
    pub pid: u32,
}

impl ProcessStatus {
    pub fn running(pid: u32) -> Self {
        Self {
            running: true,
            exit_code: None,
            pid,
        }
    }

    pub fn exited(pid: u32, exit_code: i32) -> Self {
        Self {
            running: false,
            exit_code: Some(exit_code),
            pid,
        }
    }

    pub fn is_success(&self) -> bool {
        !self.running && self.exit_code == Some(0)
    }
}
```

#### 3.5.3 ProcessError

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("process not found: pid={0}")]
    NotFound(u32),
    
    #[error("launch failed: {0}")]
    LaunchFailed(String),
    
    #[error("kill failed: {0}")]
    KillFailed(String),
    
    #[error("status check failed: {0}")]
    StatusFailed(String),
    
    #[error("permission denied: {0}")]
    PermissionDenied(String),
}

impl ProcessError {
    pub fn to_error_code(&self) -> i32 {
        match self {
            Self::NotFound(_) => -32301,
            Self::LaunchFailed(_) => -32302,
            Self::KillFailed(_) => -32303,
            Self::StatusFailed(_) => -32304,
            Self::PermissionDenied(_) => -32305,
        }
    }
}
```

### 3.6 Analysis Domain

#### 3.6.1 DiffResult

```rust
/// Result of a perceptual diff between two frames.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffResult {
    /// Whether the change ratio exceeded the requested threshold.
    pub changed: bool,
    /// Fraction of pixels that differ, in [0.0, 1.0].
    pub change_ratio: f64,
    /// Number of pixels that differ.
    pub diff_pixels: usize,
    /// Total pixels compared.
    pub total_pixels: usize,
}

impl DiffResult {
    pub fn new(threshold: f64, diff_pixels: usize, total_pixels: usize) -> Self {
        let change_ratio = diff_pixels as f64 / total_pixels as f64;
        Self {
            changed: change_ratio > threshold,
            change_ratio,
            diff_pixels,
            total_pixels,
        }
    }
}
```

#### 3.6.2 HashResult

```rust
/// Result of a content hash operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HashResult {
    /// Hex-encoded BLAKE3 hash of the normalized pixel data.
    pub hash: String,
    /// Algorithm used.
    pub algorithm: &'static str,
}

impl HashResult {
    pub fn new(hash: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            algorithm: "blake3",
        }
    }
}
```

#### 3.6.3 AnalysisError

```rust
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("decode failed: {0}")]
    DecodeFailed(String),
    
    #[error("dimension mismatch: {0}x{1} vs {2}x{3}")]
    DimensionMismatch(u32, u32, u32, u32),
    
    #[error("hash failed: {0}")]
    HashFailed(String),
    
    #[error("comparison failed: {0}")]
    ComparisonFailed(String),
}

impl AnalysisError {
    pub fn to_error_code(&self) -> i32 {
        match self {
            Self::DecodeFailed(_) => -32401,
            Self::DimensionMismatch(_, _, _, _) => -32402,
            Self::HashFailed(_) => -32403,
            Self::ComparisonFailed(_) => -32404,
        }
    }
}
```

---

## 4. Ports Layer

### 4.1 Port Design Principles

Ports define abstract capabilities that adapters implement:
- **Async by default**: All methods return `Future`
- **Thread-safe**: `Send + Sync` bounds
- **Error-specific**: Domain error types per port
- **Minimal surface**: Only essential methods

### 4.2 CapturePort

```rust
/// Port for screen capture operations.
#[async_trait]
pub trait CapturePort: Send + Sync {
    /// Capture an entire display (monitor) by index.
    /// 
    /// # Arguments
    /// * `monitor` - Zero-based monitor index (0 is primary)
    /// 
    /// # Returns
    /// * `Ok(Frame)` - Captured frame with PNG data
    /// * `Err(CaptureError::MonitorNotFound)` - Invalid monitor index
    /// * `Err(CaptureError::PermissionDenied)` - Missing screen recording permission
    /// * `Err(CaptureError::CaptureFailed)` - Platform-specific failure
    /// 
    /// # Example
    /// ```
    /// let frame = capture.capture_display(0).await?;
    /// println!("Captured {}x{} display", frame.width, frame.height);
    /// ```
    async fn capture_display(&self, monitor: u32) -> Result<Frame, CaptureError>;

    /// Capture a single window, optionally filtered by title substring.
    /// 
    /// # Arguments
    /// * `title` - Optional case-insensitive substring to match window title
    ///             If None, captures the active/focused window
    /// 
    /// # Returns
    /// * `Ok(Frame)` - Captured frame
    /// * `Err(CaptureError::WindowNotFound)` - No matching window found
    /// * `Err(CaptureError::PermissionDenied)` - Missing permissions
    /// 
    /// # Example
    /// ```
    /// let frame = capture.capture_window(Some("Chrome")).await?;
    /// ```
    async fn capture_window(&self, title: Option<&str>) -> Result<Frame, CaptureError>;

    /// List available monitors.
    /// 
    /// # Returns
    /// Vector of monitor information (index, name, resolution)
    async fn list_monitors(&self) -> Result<Vec<MonitorInfo>, CaptureError>;
}

/// Information about a monitor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonitorInfo {
    pub index: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}
```

### 4.3 InputPort

```rust
/// Port for keyboard and mouse input injection.
#[async_trait]
pub trait InputPort: Send + Sync {
    /// Press, hold, or release a keyboard key.
    /// 
    /// # Arguments
    /// * `key` - The key to actuate (e.g., "a", "return", "ctrl")
    /// * `action` - Press, Down, or Up
    /// 
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(InputError::UnknownKey)` - Key not recognized
    /// * `Err(InputError::PermissionDenied)` - Missing accessibility permission
    /// * `Err(InputError::InjectionFailed)` - Platform injection failed
    /// 
    /// # Example
    /// ```
    /// input.key_event(Key::new("ctrl"), KeyAction::Down).await?;
    /// input.key_event(Key::new("c"), KeyAction::Press).await?;
    /// input.key_event(Key::new("ctrl"), KeyAction::Up).await?;
    /// ```
    async fn key_event(&self, key: Key, action: KeyAction) -> Result<(), InputError>;

    /// Type a string of text.
    /// 
    /// This is more efficient than individual key events for text input.
    /// Handles unicode and platform-specific text entry methods.
    /// 
    /// # Arguments
    /// * `text` - The text to type
    /// 
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(InputError::InjectionFailed)` - Text entry failed
    async fn type_text(&self, text: &str) -> Result<(), InputError>;

    /// Perform a mouse event (move, click, or scroll).
    /// 
    /// # Arguments
    /// * `event` - The mouse event to perform
    /// 
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(InputError::InvalidCoordinates)` - Coordinates out of bounds
    /// * `Err(InputError::PermissionDenied)` - Missing permissions
    /// * `Err(InputError::InjectionFailed)` - Platform injection failed
    async fn mouse_event(&self, event: MouseEvent) -> Result<(), InputError>;

    /// Get the current mouse cursor position.
    /// 
    /// # Returns
    /// * `Ok((x, y))` - Current cursor coordinates
    async fn cursor_position(&self) -> Result<(i32, i32), InputError>;
}
```

### 4.4 WindowPort

```rust
/// Port for window enumeration and focus.
#[async_trait]
pub trait WindowPort: Send + Sync {
    /// List all top-level windows visible to the OS.
    /// 
    /// # Returns
    /// Vector of window information structs
    /// 
    /// # Platform Notes
    /// - Windows: Uses EnumWindows
    /// - Linux: Uses EWMH _NET_CLIENT_LIST
    /// - macOS: Uses CGWindowList + NSWorkspace
    async fn list_windows(&self) -> Result<Vec<WindowInfo>, WindowError>;

    /// Find the first window matching `filter`, or `None` if not found.
    /// 
    /// # Arguments
    /// * `filter` - Criteria for matching (title substring, pid, visibility)
    /// 
    /// # Returns
    /// * `Ok(Some(WindowInfo))` - Matching window found
    /// * `Ok(None)` - No matching window
    /// * `Err(WindowError::EnumerationFailed)` - Could not enumerate windows
    async fn find_window(&self, filter: WindowFilter) -> Result<Option<WindowInfo>, WindowError>;

    /// Bring a window to the foreground by its platform handle.
    /// 
    /// # Arguments
    /// * `hwnd` - Platform window handle (from WindowInfo.hwnd)
    /// 
    /// # Returns
    /// * `Ok(())` - Window focused
    /// * `Err(WindowError::NotFound)` - Window no longer exists
    /// * `Err(WindowError::PermissionDenied)` - Cannot focus (e.g., elevated window)
    /// 
    /// # Platform Notes
    /// - Windows: SetForegroundWindow with proper foreground rights
    /// - Linux: _NET_ACTIVE_WINDOW EWMH message
    /// - macOS: NSRunningApplication.activateWithOptions
    async fn focus_window(&self, hwnd: usize) -> Result<(), WindowError>;

    /// Get the currently focused window.
    /// 
    /// # Returns
    /// * `Ok(Some(WindowInfo))` - Active window
    /// * `Ok(None)` - No active window (e.g., desktop)
    async fn active_window(&self) -> Result<Option<WindowInfo>, WindowError>;
}
```

### 4.5 ProcessPort

```rust
/// Port for process lifecycle management.
#[async_trait]
pub trait ProcessPort: Send + Sync {
    /// Spawn a new process. Returns its PID.
    /// 
    /// # Arguments
    /// * `handle` - Process configuration (path, args, cwd, env)
    /// 
    /// # Returns
    /// * `Ok(pid)` - Process spawned successfully
    /// * `Err(ProcessError::LaunchFailed)` - Could not start process
    /// * `Err(ProcessError::PermissionDenied)` - Permission denied
    async fn launch(&self, handle: ProcessHandle) -> Result<u32, ProcessError>;

    /// Terminate a process by PID.
    /// 
    /// # Arguments
    /// * `pid` - Process ID to terminate
    /// 
    /// # Returns
    /// * `Ok(())` - Process terminated (or already terminated)
    /// * `Err(ProcessError::NotFound)` - Process not found
    /// * `Err(ProcessError::KillFailed)` - Could not terminate
    /// * `Err(ProcessError::PermissionDenied)` - Cannot terminate (e.g., system process)
    /// 
    /// # Platform Notes
    /// Sends SIGTERM on Unix, then SIGKILL if needed.
    /// Uses TerminateProcess on Windows.
    async fn kill(&self, pid: u32) -> Result<(), ProcessError>;

    /// Query whether a process is still running and its exit code if done.
    /// 
    /// # Arguments
    /// * `pid` - Process ID to query
    /// 
    /// # Returns
    /// * `Ok(ProcessStatus)` - Current process status
    /// * `Err(ProcessError::NotFound)` - Process never existed or PID recycled
    async fn status(&self, pid: u32) -> Result<ProcessStatus, ProcessError>;

    /// Wait for a process to exit.
    /// 
    /// # Arguments
    /// * `pid` - Process ID to wait for
    /// * `timeout` - Maximum time to wait
    /// 
    /// # Returns
    /// * `Ok(ProcessStatus)` - Process exited (includes exit code)
    /// * `Err(ProcessError::Timeout)` - Process still running after timeout
    async fn wait(&self, pid: u32, timeout: Duration) -> Result<ProcessStatus, ProcessError>;
}
```

### 4.6 AnalysisPort

```rust
/// Port for image analysis operations.
#[async_trait]
pub trait AnalysisPort: Send + Sync {
    /// Compute the fraction of pixels that differ between two PNG images.
    /// 
    /// # Arguments
    /// * `a` - First image (PNG bytes)
    /// * `b` - Second image (PNG bytes)
    /// * `threshold` - Threshold for "changed" determination (0.0-1.0)
    /// 
    /// # Returns
    /// * `Ok(DiffResult)` - Comparison result
    /// * `Err(AnalysisError::DecodeFailed)` - Could not decode PNG
    /// * `Err(AnalysisError::DimensionMismatch)` - Images different sizes
    async fn diff(&self, a: &[u8], b: &[u8], threshold: f32) -> Result<DiffResult, AnalysisError>;

    /// Compute a BLAKE3 hash of the normalized pixel data of a PNG image.
    /// 
    /// The normalization process:
    /// 1. Decode PNG to RGBA8
    /// 2. Resize to standard dimensions (optional)
    /// 3. Remove alpha channel
    /// 4. Compute BLAKE3 hash
    /// 
    /// # Arguments
    /// * `data` - PNG image bytes
    /// 
    /// # Returns
    /// * `Ok(HashResult)` - Hash of normalized image
    /// * `Err(AnalysisError::DecodeFailed)` - Could not decode PNG
    /// * `Err(AnalysisError::HashFailed)` - Hash computation failed
    async fn hash(&self, data: &[u8]) -> Result<HashResult, AnalysisError>;

    /// Compute a perceptual hash (pHash) for similarity comparison.
    /// 
    /// pHash is robust to minor image modifications (compression, resizing).
    /// 
    /// # Arguments
    /// * `data` - PNG image bytes
    /// 
    /// # Returns
    /// * `Ok(String)` - Hex-encoded perceptual hash
    async fn perceptual_hash(&self, data: &[u8]) -> Result<String, AnalysisError>;
}
```

---

## 5. Adapters Layer

### 5.1 Adapter Selection Strategy

Adapters are selected at compile time based on `#[cfg(target_os = ...)]`. Each platform has primary adapters with fallback to cross-platform libraries.

### 5.2 Windows Adapters

#### 5.2.1 WgcCapture (Primary)

Implements `CapturePort` using Windows Graphics Capture API.

```rust
use windows::Graphics::Capture::{GraphicsCaptureItem, Direct3D11CaptureFramePool};
use windows::Graphics::DirectX::DirectXPixelFormat;

pub struct WgcCapture {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    frame_pool: Direct3D11CaptureFramePool,
}

impl WgcCapture {
    pub fn new() -> Result<Self, CaptureError> {
        // Initialize D3D11 device
        // Create frame pool
        // Return configured capture
    }
}

#[async_trait]
impl CapturePort for WgcCapture {
    async fn capture_display(&self, monitor: u32) -> Result<Frame, CaptureError> {
        // Get GraphicsCaptureItem for monitor
        // Capture frame
        // Convert to PNG
    }

    async fn capture_window(&self, title: Option<&str>) -> Result<Frame, CaptureError> {
        // Find window handle
        // Create GraphicsCaptureItem for window
        // Capture frame
    }
}
```

**Requirements**:
- Windows 10 version 1903+
- `Windows.Graphics.Capture` API contract

**Performance**:
- 1-2ms capture latency
- GPU-accelerated
- 60 FPS capable

#### 5.2.2 SendInputAdapter (Primary)

Implements `InputPort` using Win32 SendInput API.

```rust
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, INPUT_MOUSE, 
    KEYBDINPUT, MOUSEINPUT, MOUSEEVENTF_MOVE, VIRTUAL_KEY
};

pub struct SendInputAdapter;

impl SendInputAdapter {
    pub fn new() -> Self {
        Self
    }

    fn key_to_vk(key: &Key) -> Option<VIRTUAL_KEY> {
        match key.as_str() {
            "a" => Some(VK_A),
            "b" => Some(VK_B),
            "return" => Some(VK_RETURN),
            "escape" => Some(VK_ESCAPE),
            "ctrl" => Some(VK_CONTROL),
            "shift" => Some(VK_SHIFT),
            _ => None,
        }
    }
}

#[async_trait]
impl InputPort for SendInputAdapter {
    async fn key_event(&self, key: Key, action: KeyAction) -> Result<(), InputError> {
        let vk = Self::key_to_vk(&key)
            .ok_or_else(|| InputError::UnknownKey(key.as_str().to_string()))?;

        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: match action {
                        KeyAction::Down => 0,
                        KeyAction::Up => KEYEVENTF_KEYUP,
                        KeyAction::Press => 0, // Will send down then up
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }

        if action == KeyAction::Press {
            // Send key-up
            let mut release = input;
            release.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
            unsafe {
                SendInput(&[release], std::mem::size_of::<INPUT>() as i32);
            }
        }

        Ok(())
    }

    async fn type_text(&self, text: &str) -> Result<(), InputError> {
        // Send Unicode input events for each character
    }

    async fn mouse_event(&self, event: MouseEvent) -> Result<(), InputError> {
        // Convert to MOUSEINPUT and send
    }
}
```

#### 5.2.3 EnumWindowsAdapter (Primary)

Implements `WindowPort` using Win32 EnumWindows API.

```rust
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible, SetForegroundWindow, GetWindowRect
};

pub struct EnumWindowsAdapter;

impl EnumWindowsAdapter {
    extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // Collect window info into Vec passed via lparam
    }
}

#[async_trait]
impl WindowPort for EnumWindowsAdapter {
    async fn list_windows(&self) -> Result<Vec<WindowInfo>, WindowError> {
        // Call EnumWindows with callback
        // Convert results to WindowInfo
    }

    async fn focus_window(&self, hwnd: usize) -> Result<(), WindowError> {
        let hwnd = HWND(hwnd as *mut _);
        unsafe {
            // Attach input thread
            // Set foreground window
            // Detach thread
        }
    }
}
```

### 5.3 Linux Adapters

#### 5.3.1 X11Capture (Primary)

Implements `CapturePort` using X11 XGetImage/XShm.

```rust
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{get_image, ImageFormat};

pub struct X11Capture {
    conn: RustConnection,
    screen_num: usize,
}

impl X11Capture {
    pub fn new() -> Self {
        let (conn, screen_num) = x11rb::connect(None).unwrap();
        Self { conn, screen_num }
    }
}

#[async_trait]
impl CapturePort for X11Capture {
    async fn capture_display(&self, monitor: u32) -> Result<Frame, CaptureError> {
        let screen = &self.conn.setup().roots[self.screen_num];
        let root = screen.root;

        let image = get_image(
            &self.conn,
            ImageFormat::Z_PIXMAP,
            root,
            0,
            0,
            screen.width_in_pixels as u16,
            screen.height_in_pixels as u16,
            !0,
        )
        .await
        .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

        // Convert to PNG
    }
}
```

#### 5.3.2 UinputAdapter (Primary)

Implements `InputPort` using Linux uinput.

```rust
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use nix::ioctl_write_int;

pub struct UinputAdapter {
    fd: std::fs::File,
}

impl UinputAdapter {
    pub fn new() -> Result<Self, InputError> {
        let fd = OpenOptions::new()
            .write(true)
            .open("/dev/uinput")
            .map_err(|e| InputError::InitFailed(e.to_string()))?;

        // Setup virtual device via ioctl
        // Register keys, mouse buttons

        Ok(Self { fd })
    }

    fn write_event(&self, type_: u16, code: u16, value: i32) -> Result<(), InputError> {
        let event = input_event {
            time: timeval { tv_sec: 0, tv_usec: 0 },
            type_,
            code,
            value,
        };
        // Write to fd
    }
}

#[async_trait]
impl InputPort for UinputAdapter {
    async fn key_event(&self, key: Key, action: KeyAction) -> Result<(), InputError> {
        let code = self.key_to_code(&key)?;
        let value = match action {
            KeyAction::Down => 1,
            KeyAction::Up => 0,
            KeyAction::Press => {
                self.write_event(EV_KEY, code, 1)?;
                self.write_event(EV_SYN, SYN_REPORT, 0)?;
                self.write_event(EV_KEY, code, 0)?;
                self.write_event(EV_SYN, SYN_REPORT, 0)?;
                return Ok(());
            }
        };

        self.write_event(EV_KEY, code, value)?;
        self.write_event(EV_SYN, SYN_REPORT, 0)?;
        Ok(())
    }
}
```

**Requirements**:
- `/dev/uinput` access
- udev rules or capabilities:
  ```
  KERNEL=="uinput", MODE="0660", GROUP="uinput"
  ```

#### 5.3.3 EwmhAdapter (Primary)

Implements `WindowPort` using EWMH (_NET_CLIENT_LIST).

```rust
use x11rb::protocol::ewmh;

pub struct EwmhAdapter {
    conn: RustConnection,
}

#[async_trait]
impl WindowPort for EwmhAdapter {
    async fn list_windows(&self) -> Result<Vec<WindowInfo>, WindowError> {
        let root = self.conn.setup().roots[0].root;
        let client_list = ewmh::get_client_list(&self.conn, root)
            .await
            .map_err(|e| WindowError::EnumerationFailed(e.to_string()))?;

        // Query each window for properties
        // Build WindowInfo list
    }

    async fn focus_window(&self, hwnd: usize) -> Result<(), WindowError> {
        let root = self.conn.setup().roots[0].root;
        let window = hwnd as x11rb::protocol::xproto::Window;

        // Send _NET_ACTIVE_WINDOW client message
        ewmh::request_change_active_window(
            &self.conn,
            root,
            window,
            ewmh::ClientSourceType::Normal,
            0,
            None,
        )
        .await
        .map_err(|e| WindowError::Failed(e.to_string()))?;

        Ok(())
    }
}
```

### 5.4 macOS Adapters

#### 5.4.1 CGCapture (Primary)

Implements `CapturePort` using CoreGraphics.

```rust
use core_graphics::display::{
    CGDisplay, CGDisplayBounds, CGDisplayCreateImage,
    CGWindowListCreateImage, kCGWindowListOptionIncludingWindow
};
use core_graphics::image::CGImage;

pub struct CGCapture;

impl CGCapture {
    pub fn new() -> Self {
        Self
    }

    fn cgimage_to_png(cg_image: CGImage) -> Result<Vec<u8>, CaptureError> {
        // Use CGImageDestination to encode as PNG
    }
}

#[async_trait]
impl CapturePort for CGCapture {
    async fn capture_display(&self, monitor: u32) -> Result<Frame, CaptureError> {
        let display_id = monitor as CGDirectDisplayID;
        let cg_image = CGDisplay::screenshot(display_id, None, None, None)
            .ok_or_else(|| CaptureError::CaptureFailed("screenshot failed".to_string()))?;

        let png_data = Self::cgimage_to_png(cg_image)?;
        let bounds = CGDisplayBounds(display_id);

        Ok(Frame::new(
            BASE64.encode(&png_data),
            bounds.size.width as u32,
            bounds.size.height as u32,
        ))
    }

    async fn capture_window(&self, title: Option<&str>) -> Result<Frame, CaptureError> {
        // Find window by title using CGWindowList
        // Capture specific window
    }
}
```

**Requirements**:
- Screen Recording permission (TCC)
- CGPreflightScreenCaptureAccess() returns true

#### 5.4.2 CGEventAdapter (Primary)

Implements `InputPort` using CoreGraphics CGEvent.

```rust
use core_graphics::event::{CGEvent, CGEventType, CGMouseButton};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;

pub struct CGEventAdapter {
    source: CGEventSource,
}

impl CGEventAdapter {
    pub fn new() -> Result<Self, InputError> {
        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .map_err(|e| InputError::InitFailed(e.to_string()))?;
        Ok(Self { source })
    }

    fn key_to_cgkey(key: &Key) -> Option<CGKeyCode> {
        match key.as_str() {
            "a" => Some(0),
            "b" => Some(11),
            "return" => Some(36),
            "escape" => Some(53),
            _ => None,
        }
    }
}

#[async_trait]
impl InputPort for CGEventAdapter {
    async fn key_event(&self, key: Key, action: KeyAction) -> Result<(), InputError> {
        let keycode = Self::key_to_cgkey(&key)
            .ok_or_else(|| InputError::UnknownKey(key.as_str().to_string()))?;

        let event = CGEvent::new_keyboard_event(self.source.clone(), keycode, action != KeyAction::Up)
            .map_err(|e| InputError::InjectionFailed(e.to_string()))?;

        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    async fn mouse_event(&self, event: MouseEvent) -> Result<(), InputError> {
        match event {
            MouseEvent::Move { x, y } => {
                let point = CGPoint::new(x as f64, y as f64);
                let event = CGEvent::new_mouse_event(
                    self.source.clone(),
                    CGEventType::MouseMoved,
                    point,
                    CGMouseButton::Left,
                )
                .map_err(|e| InputError::InjectionFailed(e.to_string()))?;
                event.post(CGEventTapLocation::HID);
            }
            // ... handle other event types
        }
        Ok(())
    }
}
```

**Requirements**:
- Accessibility permission (TCC)

#### 5.4.3 NSWorkspaceAdapter (Primary)

Implements `WindowPort` using NSWorkspace + CGWindowList.

```rust
use cocoa::appkit::NSWorkspace;
use cocoa::base::{id, nil};
use cocoa::foundation::NSString;

pub struct NSWorkspaceAdapter;

impl NSWorkspaceAdapter {
    fn get_window_list() -> Vec<WindowInfo> {
        // Use CGWindowListCopyWindowInfo
        // Filter for on-screen windows
        // Build WindowInfo list
    }
}

#[async_trait]
impl WindowPort for NSWorkspaceAdapter {
    async fn list_windows(&self) -> Result<Vec<WindowInfo>, WindowError> {
        Ok(Self::get_window_list())
    }

    async fn focus_window(&self, hwnd: usize) -> Result<(), WindowError> {
        // Find NSRunningApplication by PID
        // Call activateWithOptions
    }
}
```

### 5.5 Cross-Platform Fallbacks

#### 5.5.1 XcapCapture

Implements `CapturePort` using xcap library.

```rust
pub struct XcapCapture;

impl XcapCapture {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CapturePort for XcapCapture {
    async fn capture_display(&self, monitor: u32) -> Result<Frame, CaptureError> {
        let monitors = xcap::Monitor::all()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

        let monitor = monitors.get(monitor as usize)
            .ok_or(CaptureError::MonitorNotFound(monitor))?;

        let image = monitor.capture_image()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

        // Convert image to PNG + base64
    }
}
```

#### 5.5.2 EnigoInput

Implements `InputPort` using enigo library.

```rust
pub struct EnigoInput {
    enigo: enigo::Enigo,
}

impl EnigoInput {
    pub fn new() -> Self {
        Self {
            enigo: enigo::Enigo::new(),
        }
    }
}

#[async_trait]
impl InputPort for EnigoInput {
    async fn key_event(&self, key: Key, action: KeyAction) -> Result<(), InputError> {
        let enigo_key = self.convert_key(&key)?;
        
        match action {
            KeyAction::Press => self.enigo.key_click(enigo_key),
            KeyAction::Down => self.enigo.key_down(enigo_key),
            KeyAction::Up => self.enigo.key_up(enigo_key),
        }
        
        Ok(())
    }

    async fn type_text(&self, text: &str) -> Result<(), InputError> {
        self.enigo.key_sequence(text);
        Ok(())
    }

    async fn mouse_event(&self, event: MouseEvent) -> Result<(), InputError> {
        // Convert to enigo calls
    }
}
```

### 5.6 Noop Stub

For unknown platforms, noop stubs return errors:

```rust
struct NoopWindowAdapter;

#[async_trait]
impl WindowPort for NoopWindowAdapter {
    async fn list_windows(&self) -> Result<Vec<WindowInfo>, WindowError> {
        Ok(vec![])
    }

    async fn find_window(&self, _: WindowFilter) -> Result<Option<WindowInfo>, WindowError> {
        Ok(None)
    }

    async fn focus_window(&self, _: usize) -> Result<(), WindowError> {
        Err(WindowError::Failed("not supported on this platform".to_string()))
    }

    async fn active_window(&self) -> Result<Option<WindowInfo>, WindowError> {
        Ok(None)
    }
}
```

---

## 6. IPC Layer

### 6.1 JSON-RPC 2.0 Implementation

#### 6.1.1 Request Structure

```rust
/// JSON-RPC 2.0 request from caller.
#[derive(Debug, Deserialize)]
pub struct Request {
    /// Must be exactly "2.0"
    pub jsonrpc: String,
    
    /// Request identifier (Number, String, or Null)
    /// Used to correlate requests with responses
    pub id: serde_json::Value,
    
    /// Method name to invoke
    pub method: String,
    
    /// Optional parameters (object, array, or omitted)
    pub params: Option<serde_json::Value>,
}

impl Request {
    /// Validate the request is well-formed.
    pub fn validate(&self) -> Result<(), String> {
        if self.jsonrpc != "2.0" {
            return Err("invalid jsonrpc version".to_string());
        }
        if self.method.is_empty() {
            return Err("method is required".to_string());
        }
        Ok(())
    }
}
```

#### 6.1.2 Response Structure

```rust
/// JSON-RPC 2.0 response to caller.
#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Response {
    /// Successful response.
    pub fn ok(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Error response.
    pub fn err(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    pub fn method_not_found(id: serde_json::Value, method: &str) -> Self {
        Self::err(id, -32601, format!("Method not found: {method}"))
    }

    pub fn invalid_params(id: serde_json::Value, msg: impl Into<String>) -> Self {
        Self::err(id, -32602, msg)
    }

    pub fn internal_error(id: serde_json::Value, msg: impl Into<String>) -> Self {
        Self::err(id, -32603, msg)
    }
}
```

#### 6.1.3 Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON was received |
| -32600 | Invalid Request | The JSON sent is not a valid Request object |
| -32601 | Method not found | The method does not exist |
| -32602 | Invalid params | Invalid method parameter(s) |
| -32603 | Internal error | Internal JSON-RPC error |
| -32000 to -32099 | Server error | Reserved for implementation-defined errors |
| -32001 | Window not found | Window matching criteria not found |
| -32002 | Monitor not found | Monitor index invalid |
| -32003 | Capture failed | Screen capture operation failed |
| -32101 | Unknown key | Key name not recognized |
| -32102 | Injection failed | Input injection failed |
| -32104 | Permission denied | Missing required permission |

### 6.2 Wire Protocol

#### 6.2.1 Transport

bare-cua uses newline-delimited JSON (NDJSON) over standard streams:

```
stdin  ──▶  Requests (JSON-RPC)
stdout ──▶  Responses (JSON-RPC)
stderr ──▶  Logs (structured JSON)
```

#### 6.2.2 Reading Requests

```rust
/// Read one newline-delimited JSON-RPC request from stdin.
pub async fn read_request<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
) -> Result<Option<Request>> {
    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    
    if n == 0 {
        return Ok(None); // EOF
    }

    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let req: Request = serde_json::from_str(trimmed)?;
    req.validate()?;
    Ok(Some(req))
}
```

#### 6.2.3 Writing Responses

```rust
/// Write one JSON-RPC response to stdout, followed by newline.
pub async fn write_response<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    response: &Response,
) -> Result<()> {
    let mut json = serde_json::to_string(response)?;
    json.push('\n');
    writer.write_all(json.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}
```

#### 6.2.4 Structured Logging

```rust
// Initialize tracing to stderr in JSON format
tracing_subscriber::fmt()
    .with_writer(std::io::stderr)
    .with_env_filter(
        EnvFilter::from_env("BARE_CUA_LOG")
            .add_directive("bare_cua_native=info".parse().unwrap()),
    )
    .json()
    .init();

// Usage
info!(version = env!("CARGO_PKG_VERSION"), "bare-cua-native starting");
error!(error = %e, "Failed to parse request");
```

### 6.3 Dispatcher

#### 6.3.1 Method Routing

```rust
pub struct Dispatcher {
    pub capture: Arc<dyn CapturePort>,
    pub input: Arc<dyn InputPort>,
    pub windows: Arc<dyn WindowPort>,
    pub process: Arc<dyn ProcessPort>,
    pub analysis: Arc<dyn AnalysisPort>,
    pub plugins: PluginRegistry,
}

impl Dispatcher {
    #[instrument(name = "dispatcher.dispatch", skip(self), fields(method = %req.method))]
    pub async fn dispatch(&self, req: Request) -> Response {
        let id = req.id.clone();
        let params = req.params.unwrap_or(Value::Null);

        match req.method.as_str() {
            // Health
            "ping" => Response::ok(id, json!({ "ok": true, "version": env!("CARGO_PKG_VERSION") })),

            // Capture
            "screenshot" => self.handle_screenshot(id, params).await,

            // Input
            "input.key" => self.handle_input_key(id, params).await,
            "input.type" => self.handle_input_type(id, params).await,
            "input.click" => self.handle_input_click(id, params).await,
            "input.scroll" => self.handle_input_scroll(id, params).await,
            "input.move" => self.handle_input_move(id, params).await,

            // Windows
            "windows.list" => self.handle_windows_list(id).await,
            "windows.focus" => self.handle_windows_focus(id, params).await,
            "windows.find" => self.handle_windows_find(id, params).await,

            // Process
            "process.launch" => self.handle_process_launch(id, params).await,
            "process.kill" => self.handle_process_kill(id, params).await,
            "process.status" => self.handle_process_status(id, params).await,

            // Analysis
            "analysis.diff" => self.handle_analysis_diff(id, params).await,
            "analysis.hash" => self.handle_analysis_hash(id, params).await,

            // Plugins
            unknown => {
                if let Some(plugin) = self.plugins.find(unknown) {
                    match plugin.handle(params).await {
                        Ok(result) => Response::ok(id, result),
                        Err(e) => Response::internal_error(id, e.to_string()),
                    }
                } else {
                    Response::method_not_found(id, unknown)
                }
            }
        }
    }
}
```

#### 6.3.2 Handler Implementations

Screenshot handler example:

```rust
async fn handle_screenshot(&self, id: Value, params: Value) -> Response {
    #[derive(serde::Deserialize, Default)]
    struct P {
        window_title: Option<String>,
        monitor: Option<u32>,
    }

    let p: P = match deserialize_or_default(params) {
        Ok(v) => v,
        Err(e) => return Response::invalid_params(id, e),
    };

    let result = if let Some(ref title) = p.window_title {
        self.capture.capture_window(Some(title.as_str())).await
    } else {
        self.capture.capture_display(p.monitor.unwrap_or(0)).await
    };

    match result {
        Ok(frame) => Response::ok(id, json!({
            "data": frame.data,
            "width": frame.width,
            "height": frame.height,
            "format": "png",
        })),
        Err(e) => Response::internal_error(id, e.to_string()),
    }
}
```

---

## 7. Application Layer

### 7.1 Dependency Injection

The app layer wires all components together:

```rust
pub struct App {
    pub dispatcher: Dispatcher,
}

impl App {
    pub fn build() -> Self {
        let capture: Arc<dyn CapturePort> = build_capture();
        let input: Arc<dyn InputPort> = build_input();
        let windows: Arc<dyn WindowPort> = build_window();
        let process: Arc<dyn ProcessPort> = Arc::new(NativeProcessAdapter::new());
        let analysis: Arc<dyn AnalysisPort> = Arc::new(NativeAnalysisAdapter::new());

        App {
            dispatcher: Dispatcher::new(
                capture, input, windows, process, analysis,
                PluginRegistry::new()
            ),
        }
    }
}
```

### 7.2 Platform Selection

```rust
// Windows
#[cfg(target_os = "windows")]
fn build_capture() -> Arc<dyn CapturePort> {
    use crate::adapters::windows::wgc::WgcCapture;
    use crate::adapters::xcap::XcapCapture;

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

// Linux
#[cfg(target_os = "linux")]
fn build_capture() -> Arc<dyn CapturePort> {
    use crate::adapters::linux::x11capture::X11Capture;
    Arc::new(X11Capture::new())
}

// macOS
#[cfg(target_os = "macos")]
fn build_capture() -> Arc<dyn CapturePort> {
    use crate::adapters::macos::cgcapture::CGCapture;
    Arc::new(CGCapture::new())
}
```

---

## 8. Plugin System

### 8.1 Plugin Trait

```rust
/// A plugin that handles a single JSON-RPC method name.
#[async_trait]
pub trait MethodPlugin: Send + Sync {
    /// The exact method name this plugin handles (e.g. "custom.foo").
    fn method_name(&self) -> &'static str;

    /// Handle an incoming request.
    async fn handle(&self, params: Value) -> anyhow::Result<Value>;
}
```

### 8.2 Plugin Registry

```rust
pub struct PluginRegistry {
    plugins: Vec<Box<dyn MethodPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn MethodPlugin>) {
        if let Some(pos) = self.plugins.iter().position(|p| p.method_name() == plugin.method_name()) {
            self.plugins[pos] = plugin; // Replace existing
        } else {
            self.plugins.push(plugin);
        }
    }

    pub fn find(&self, method: &str) -> Option<&dyn MethodPlugin> {
        self.plugins
            .iter()
            .find(|p| p.method_name() == method)
            .map(|p| p.as_ref())
    }
}
```

### 8.3 Example Plugin

```rust
struct EchoPlugin;

#[async_trait]
impl MethodPlugin for EchoPlugin {
    fn method_name(&self) -> &'static str {
        "echo"
    }

    async fn handle(&self, params: Value) -> anyhow::Result<Value> {
        Ok(params)
    }
}

// Registration
let mut registry = PluginRegistry::new();
registry.register(Box::new(EchoPlugin));
```

---

## 9. API Reference

### 9.1 Method Summary

| Method | Description | Category |
|--------|-------------|----------|
| `ping` | Health check | Meta |
| `screenshot` | Capture screen/window | Capture |
| `input.key` | Keyboard key event | Input |
| `input.type` | Type text | Input |
| `input.click` | Mouse click | Input |
| `input.scroll` | Mouse scroll | Input |
| `input.move` | Mouse move | Input |
| `windows.list` | List windows | Windows |
| `windows.focus` | Focus window | Windows |
| `windows.find` | Find window | Windows |
| `process.launch` | Spawn process | Process |
| `process.kill` | Kill process | Process |
| `process.status` | Query process | Process |
| `analysis.diff` | Compare images | Analysis |
| `analysis.hash` | Hash image | Analysis |

### 9.2 ping

Health check returning version.

**Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"ping"}
```

**Response:**
```json
{"jsonrpc":"2.0","id":1,"result":{"ok":true,"version":"0.1.0"}}
```

### 9.3 screenshot

Capture a screenshot of a monitor or window.

**Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"screenshot","params":{"monitor":0}}
```

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| monitor | integer | No | 0 | Zero-based monitor index |
| window_title | string | No | null | Window title substring |

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "data": "iVBORw0KGgo...",
    "width": 1920,
    "height": 1080,
    "format": "png"
  }
}
```

**Errors:**
- `-32002`: Monitor not found
- `-32001`: Window not found
- `-32005`: Permission denied

### 9.4 input.key

Press, hold, or release a keyboard key.

**Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"input.key","params":{"key":"return","action":"press"}}
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| key | string | Yes | Key name ("a", "return", "escape", "ctrl", etc.) |
| action | string | Yes | "press", "down", or "up" |

**Response:**
```json
{"jsonrpc":"2.0","id":1,"result":{"ok":true}}
```

### 9.5 input.type

Type a string of text.

**Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"input.type","params":{"text":"Hello, World!"}}
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| text | string | Yes | Text to type |

### 9.6 input.click

Perform a mouse click.

**Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"input.click","params":{"x":100,"y":200,"button":"left","action":"click"}}
```

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| x | integer | Yes | - | X coordinate |
| y | integer | Yes | - | Y coordinate |
| button | string | Yes | - | "left", "right", "middle" |
| action | string | Yes | - | "click", "down", "up" |

### 9.7 windows.list

List all top-level windows.

**Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"windows.list"}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": [
    {
      "hwnd": 12345,
      "title": "Google Chrome",
      "pid": 1234,
      "x": 0,
      "y": 0,
      "width": 1920,
      "height": 1080,
      "visible": true
    }
  ]
}
```

### 9.8 process.launch

Spawn a new process.

**Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"process.launch","params":{"path":"/usr/bin/ls","args":["-la"],"cwd":"/tmp"}}
```

**Response:**
```json
{"jsonrpc":"2.0","id":1,"result":{"pid":12345}}
```

### 9.9 analysis.diff

Compare two images.

**Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"analysis.diff","params":{"image_a":"base64...","image_b":"base64...","threshold":0.02}}
```

**Response:**
```json
{"jsonrpc":"2.0","id":1,"result":{"changed":true,"change_ratio":0.15}}
```

---

## 10. Platform-Specific Details

### 10.1 Windows

#### 10.1.1 Permissions

- **No special permissions** for basic operation
- **UIPI**: Cannot inject input to elevated processes from non-elevated
- **WGC**: Windows 10 1903+ required

#### 10.1.2 Build Configuration

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
    "Graphics_Capture",
    "Graphics_DirectX_Direct3D11",
] }
```

### 10.2 Linux

#### 10.2.1 Permissions

- **uinput**: Requires `/dev/uinput` access
- Setup: `sudo usermod -aG uinput $USER`
- Or: `sudo setcap cap_sys_admin+ep ./bare-cua-native`

#### 10.2.2 X11 vs Wayland

| Feature | X11 | Wayland |
|---------|-----|---------|
| Capture | ✅ XGetImage | 🔶 PipeWire portal |
| Input | ✅ uinput/XTest | 🔶 uinput only |
| Windows | ✅ EWMH | ❌ No standard |

### 10.3 macOS

#### 10.3.1 TCC Permissions

Required entitlements:
- **Screen Recording**: for capture
- **Accessibility**: for input

Runtime check:
```rust
#[cfg(target_os = "macos")]
fn check_permissions() -> bool {
    unsafe {
        CGPreflightScreenCaptureAccess() &&
        AXIsProcessTrustedWithOptions(None)
    }
}
```

---

## 11. Performance Characteristics

### 11.1 Benchmarks

| Operation | Target | Typical |
|-----------|--------|---------|
| Screenshot (cold) | < 500ms | 200ms |
| Screenshot (warm) | < 50ms | 15ms |
| Key press | < 10ms | 1ms |
| Mouse move | < 10ms | 1ms |
| Window list | < 100ms | 20ms |
| Process launch | < 50ms | 10ms |
| JSON-RPC overhead | < 1ms | 0.05ms |

### 11.2 Memory Usage

| Component | Target |
|-----------|--------|
| Base binary | < 10MB |
| Runtime (idle) | < 20MB |
| Screenshot buffer | < 10MB (1080p PNG) |
| Total typical | < 50MB |

### 11.3 Optimization Strategies

1. **Reuse capture buffers**: Avoid reallocating for each screenshot
2. **Connection pooling**: Reuse OS connections (X11, etc.)
3. **Async I/O**: All operations non-blocking
4. **Zero-copy where possible**: Minimize buffer copies

---

## 12. Security Model

### 12.1 Threat Model

| Threat | Risk | Mitigation |
|--------|------|------------|
| Unauthorized capture | High | OS permissions (TCC, etc.) |
| Unauthorized input | High | OS permissions |
| Keylogging | Medium | User approval required |
| Process escape | Low | Standard process isolation |
| JSON injection | Low | Input validation |

### 12.2 Permissions by Platform

| Platform | Capture | Input | Windows | Process |
|----------|---------|-------|---------|---------|
| macOS | Screen Recording | Accessibility | None | None |
| Linux | None | /dev/uinput | None | None |
| Windows | None | None | None | None |

### 12.3 Security Recommendations

1. **Run with minimal permissions**
2. **Audit all code** (supply chain security)
3. **Use isolated environment** for untrusted scripts
4. **Monitor stderr logs** for suspicious activity

---

## 13. Testing Strategy

### 13.1 Test Levels

| Level | Type | Scope |
|-------|------|-------|
| Unit | Domain logic | Pure functions |
| Integration | Adapter + Mock | Port implementations |
| E2E | Full system | Real platform APIs |

### 13.2 Mock Adapters

```rust
pub struct MockCapturePort {
    frames: Vec<Frame>,
}

#[async_trait]
impl CapturePort for MockCapturePort {
    async fn capture_display(&self, monitor: u32) -> Result<Frame, CaptureError> {
        self.frames.get(monitor as usize)
            .cloned()
            .ok_or(CaptureError::MonitorNotFound(monitor))
    }
    // ...
}
```

### 13.3 Test Commands

```bash
# Run all tests
cargo test --workspace

# Run with tracing
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_screenshot -- --nocapture
```

---

## 14. Deployment

### 14.1 Building

```bash
# Native binary
cargo build --release

# Cross-compile (requires appropriate toolchain)
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target x86_64-apple-darwin
```

### 14.2 Distribution

| Format | Command | Notes |
|--------|---------|-------|
| Binary | `cargo install` | From crates.io |
| Archive | `tar.gz` / `zip` | Manual distribution |
| Package | Homebrew, APT, etc. | Platform packages |

### 14.3 Container Deployment

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libx11-6
COPY --from=builder /app/target/release/bare-cua-native /usr/local/bin/
ENTRYPOINT ["bare-cua-native"]
```

---

## 15. Extensibility

### 15.1 Plugin Development

```rust
use bare_cua_native::plugins::MethodPlugin;

pub struct MyPlugin;

#[async_trait]
impl MethodPlugin for MyPlugin {
    fn method_name(&self) -> &'static str {
        "myextension.action"
    }

    async fn handle(&self, params: Value) -> anyhow::Result<Value> {
        // Implementation
        Ok(json!({ "result": "success" }))
    }
}
```

### 15.2 Custom Adapters

Implement a port trait for custom hardware or virtualization:

```rust
pub struct VncCapturePort {
    client: vnc::Client,
}

#[async_trait]
impl CapturePort for VncCapturePort {
    async fn capture_display(&self, monitor: u32) -> Result<Frame, CaptureError> {
        // Capture from VNC
    }
}
```

---

## 16. Future Work

### 16.1 Short-term (6 months)

- [ ] Wayland native capture (PipeWire)
- [ ] Hardware cursor capture
- [ ] Video recording (not just screenshots)
- [ ] OCR integration
- [ ] Element detection (CV-based)

### 16.2 Medium-term (12 months)

- [ ] Browser integration (CDP)
- [ ] Accessibility tree access
- [ ] Mobile support (iOS/Android via USB)
- [ ] WASM plugin runtime
- [ ] External process plugins

### 16.3 Long-term (24 months)

- [ ] AI-native architecture (built-in VLM)
- [ ] Distributed agents (multi-machine)
- [ ] Cloud deployment (containerized)
- [ ] Enterprise features (audit logging, RBAC)

---

## 17. Appendices

### Appendix A: OpenRPC Schema

See `contracts/openrpc.json` for machine-readable API specification.

### Appendix B: Error Code Reference

See Section 6.1.3 for complete error code listing.

### Appendix C: Architecture Decision Records

See `docs/adr/` for detailed architectural decisions:
- ADR-001: Hexagonal Architecture with JSON-RPC 2.0 IPC
- ADR-002: Platform Adapter Selection Strategy
- ADR-003: Plugin System Architecture

### Appendix D: Changelog

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-04-04 | Initial release |

---

*End of Specification*

@trace BCUA-SPEC-001
