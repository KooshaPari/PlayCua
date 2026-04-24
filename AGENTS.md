# AGENTS.md — PlayCua

## Project Overview

- **Name**: PlayCua (CUA - Conversational User Agent)
- **Description**: Conversational user agent framework with cross-platform screen capture and interaction
- **Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/PlayCua`
- **Language Stack**: Rust, Protocol Buffers, gRPC
- **Published**: Private (Phenotype org)

## Quick Start

```bash
# Navigate to project
cd /Users/kooshapari/CodeProjects/Phenotype/repos/PlayCua

# Build
cargo build

# Run tests
cargo test

# Run example
cargo run --example capture
```

## Architecture

### CUA Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Conversation Engine                            │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │                    LLM Interface                              │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │ │
│  │  │ Context    │  │ Prompt     │  │ Response   │         │ │
│  │  │ Manager    │  │ Builder    │  │ Parser     │         │ │
│  │  └────────────┘  └────────────┘  └────────────┘         │ │
│  └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Screen Capture (Native)                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   macOS         │  │   Windows       │  │   Linux         │ │
│  │   (CoreGraphics)│  │   (DXGI)        │  │   (PipeWire)    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Interaction Layer                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐│
│  │   Mouse           │  │   Keyboard      │  │   Touch         ││
│  │   Control         │  │   Input         │  │   (Mobile)      ││
│  └─────────────────┘  └─────────────────┘  └─────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Quality Standards

### Rust Quality

- **Formatter**: rustfmt
- **Linter**: clippy
- **Tests**: cargo test
- **Safety**: Careful with unsafe OS APIs

## Git Workflow

### Branch Naming

Format: `<type>/<platform>/<description>`

Examples:
- `feat/macos/add-window-capture`
- `fix/windows/handle-dpi`
- `feat/protocol/add-streaming`

## CLI Commands

```bash
cargo build
cargo test
cargo clippy --all-targets
```

## Resources

- [CoreGraphics](https://developer.apple.com/documentation/coregraphics)
- [DXGI](https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/)
- [Phenotype Registry](https://github.com/KooshaPari/phenotype-registry)

## Agent Notes

**Critical Details:**
- Platform-specific capture APIs
- Privacy permissions required
- Performance optimization
- Memory management

**Known Gotchas:**
- Permissions dialogs on macOS
- GPU vs CPU capture tradeoffs
- HiDPI screen handling
- Cross-platform differences
