# AGENTS.md — bare-cua

## Project Overview

- **Name**: bare-cua
- **Description**: Barebones Computer Use Agent Framework — Minimal CUA implementation with Rust core for agent-based automation and UI interaction
- **Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/bare-cua`
- **Language Stack**: Rust (Edition 2024), Python 3.12+ (bindings)
- **Published**: Private (Phenotype org)

## Quick Start Commands

```bash
# Clone and setup
git clone https://github.com/KooshaPari/bare-cua.git
cd bare-cua

# Install Rust toolchain
rustup update nightly
rustup default nightly

# Build Rust core
cargo build --release

# Run tests
cargo test

# Build Python bindings
cd bindings/python
pip install maturin
maturin develop

# Run example
python examples/basic.py
```

## Architecture

### CUA Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Client Interface Layer                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   Python        │  │   Rust          │  │   CLI           │         │
│  │   Bindings      │  │   Library       │  │   Tool          │         │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘         │
└───────────┼────────────────────┼────────────────────┼────────────────┘
            │                    │                    │
            ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Core Engine (Rust)                                │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    bare-cua Core                               │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐            │   │
│  │  │   Action   │  │   Vision   │  │   Input    │            │   │
│  │  │   Executor │  │   Parser   │  │   Handler  │            │   │
│  │  └────────────┘  └────────────┘  └────────────┘            │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐            │   │
│  │  │   State    │  │   Plan     │  │   Memory   │            │   │
│  │  │   Manager  │  │   Generator│  │   Store    │            │   │
│  │  └────────────┘  └────────────┘  └────────────┘            │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Platform Adapters                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   macOS         │  │   Linux         │  │   Windows       │         │
│  │   (AX)          │  │   (X11/Wayland) │  │   (Win32)       │         │
│  │                 │  │                 │  │                 │         │
│  │  • Accessibility│  │  • AT-SPI       │  │  • UIAutomation │         │
│  │  • Screenshot   │  │  • XTest        │  │  • SendInput    │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
└─────────────────────────────────────────────────────────────────────┘
```

### Action Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CUA Action Execution Flow                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐      │
│  │  Observe │───▶│  Think   │───▶│  Act     │───▶│  Verify  │      │
│  │          │    │          │    │          │    │          │      │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘      │
│       │               │               │               │             │
│       ▼               ▼               ▼               ▼             │
│  Screenshot      LLM Analysis    Execute Action   Check Result      │
│  UI Tree         Plan Steps      Click/Type       Success/Retry      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Quality Standards

### Rust Code Quality

- **Formatter**: `rustfmt` (nightly)
- **Linter**: `clippy --all-targets --all-features -- -D warnings`
- **Type Safety**: `#![deny(unsafe_code)]` where possible
- **Tests**: `cargo nextest run` with coverage >80%

### Python Code Quality (Bindings)

- **Formatter**: `ruff format`
- **Linter**: `ruff check`
- **Type Checker**: `mypy --strict`
- **Tests**: `pytest` with coverage >75%

### Test Requirements

```bash
# Rust tests
cargo test
cargo nextest run

# Python tests
cd bindings/python
pytest

# Integration tests
cargo test --test integration

# Benchmarks
cargo bench
```

## Git Workflow

### Branch Naming

Format: `<type>/<component>/<description>`

Types: `feat`, `fix`, `docs`, `refactor`, `perf`

Examples:
- `feat/core/add-vision-parser`
- `fix/macos/accessibility-permissions`
- `refactor/executor/extract-trait`
- `perf/screenshot/use-gpu-texture`

### Commit Messages

Format: `<type>(<scope>): <description>`

Examples:
- `feat(core): implement screenshot capture with GPU acceleration`
- `fix(macos): handle accessibility permissions gracefully`
- `refactor(executor): extract action trait for testability`
- `docs(bindings): add Python API reference`

## File Structure

```
bare-cua/
├── src/
│   ├── lib.rs              # Library root
│   ├── core/               # Core engine
│   │   ├── action.rs       # Action types
│   │   ├── executor.rs     # Action executor
│   │   ├── vision.rs       # Vision/Screen parsing
│   │   ├── state.rs        # State management
│   │   └── planner.rs      # Action planning
│   ├── platform/           # Platform implementations
│   │   ├── macos.rs        # macOS adapter
│   │   ├── linux.rs        # Linux adapter
│   │   ├── windows.rs      # Windows adapter
│   │   └── traits.rs       # Platform traits
│   └── bindings/           # Language bindings
│       └── python.rs         # PyO3 bindings
├── bindings/
│   └── python/             # Python package
│       ├── src/
│       └── tests/
├── benches/                # Benchmarks
├── tests/                  # Integration tests
└── examples/               # Usage examples
```

## CLI Commands

### Development

```bash
# Build
cargo build

# Build release
cargo build --release

# Run with logging
RUST_LOG=debug cargo run

# Check
cargo check

# Format
cargo fmt

# Lint
cargo clippy -- -D warnings
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Nextest (preferred)
cargo nextest run

# Benchmark
cargo bench
```

### Python Bindings

```bash
# Setup Python environment
cd bindings/python
python -m venv venv
source venv/bin/activate

# Install maturin
pip install maturin

# Build and install
maturin develop

# Run tests
pytest

# Build wheel
maturin build --release
```

### Examples

```bash
# Basic automation
cargo run --example basic

# With vision
cargo run --example vision

# Multi-step task
cargo run --example workflow

# Python example
python bindings/python/examples/basic.py
```

## Troubleshooting

### macOS Accessibility Permissions

```bash
# Grant permissions
# 1. Open System Settings > Privacy & Security > Accessibility
# 2. Add your terminal/IDE
# 3. Restart terminal

# Verify permissions
osascript -e 'tell application "System Events" to get name of first application process'
```

### Linux X11/Wayland Issues

```bash
# X11 - install dependencies
sudo apt-get install libx11-dev libxtst-dev libxinerama-dev

# Wayland - use compatibility mode
export GDK_BACKEND=x11
cargo run

# Check display
echo $DISPLAY
```

### Windows UI Automation

```bash
# Enable UI Automation (should be on by default)
# If issues, check Windows settings

# Run as administrator for some actions
cargo run
```

### Build Failures

```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update

# Check lockfile
rm Cargo.lock
cargo build
```

### Python Binding Issues

```bash
# Rebuild bindings
cd bindings/python
maturin develop --release

# Check Python version compatibility
python --version  # Requires 3.8+

# Clear cache
rm -rf __pycache__ *.so
maturin develop
```

## Environment Variables

```bash
# Logging
RUST_LOG=info  # error, warn, info, debug, trace

# Platform
CUA_PLATFORM=auto  # auto, macos, linux, windows

# Performance
CUA_GPU_ACCELERATION=1
CUA_SCREENSHOT_QUALITY=high

# Safety
CUA_CONFIRM_DANGEROUS=1
CUA_MAX_ACTIONS=100
```

## Integration Points

| System | Protocol | Purpose |
|--------|----------|---------|
| PhenoMCP | Rust API | Agent integration |
| HeliosApp | FFI | UI automation |
| TheGent | Python API | Task scripting |
| Portage | gRPC | CI/CD automation |

## AgilePlus Integration

All work MUST be tracked in AgilePlus:
- Reference: `.agileplus/` directory
- CLI: `agileplus <command>` (from project root)
- Specs: `.agileplus/specs/<feature-id>/`

## Governance Rules

### Key Constraints

- Keep core minimal (<10k lines Rust)
- Zero dependencies in hot path
- Platform abstraction via traits
- Memory safety: no unsafe in core

### Quality Gates

- `cargo clippy -- -D warnings` — 0 warnings required
- `cargo test` — all pass required
- `cargo bench` — no performance regressions

---

Last Updated: 2026-04-05
Version: 1.0.0
