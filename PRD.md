# PRD — bare-cua

## Overview

bare-cua is a cross-platform Computer-Use Agent (CUA) framework that enables AI agents and automation scripts to interact with desktop environments programmatically. It provides native screen capture, input injection, window management, and process control through a language-agnostic JSON-RPC 2.0 interface.

## Vision

bare-cua aspires to be the **definitive open-source infrastructure for computer-use automation**—the layer that bridges AI perception (screenshots, vision models) with action (keyboard, mouse, process execution) across all major desktop platforms. We envision a world where:

- AI agents can autonomously operate any desktop application without platform-specific code
- Developers build once on bare-cua and deploy to macOS, Linux, and Windows
- The framework is fast, secure, and embeddable in any Rust, Python, or JavaScript project
- The plugin ecosystem enables specialized automation for every domain

## Epics

### E1 — Core Platform Support

| Story | Description | Acceptance Criteria |
|-------|-------------|---------------------|
| E1.1 | Windows Graphics Capture integration | `screenshot` returns PNG data from WGC API on Windows 10 1903+ |
| E1.2 | macOS CoreGraphics capture | `screenshot` returns PNG data with TCC Screen Recording permission |
| E1.3 | Linux X11 capture | `screenshot` returns PNG data via XGetImage on X11 systems |
| E1.4 | Cross-platform input injection | `input.key`, `input.click`, `input.scroll` work identically across platforms |
| E1.5 | Window enumeration and focus | `windows.list` returns all visible windows; `windows.focus` activates target |

### E2 — Performance Targets

| Story | Description | Acceptance Criteria |
|-------|-------------|---------------------|
| E2.1 | Cold screenshot latency | Screenshot completes in <500ms from request to first byte |
| E2.2 | Warm screenshot latency | Screenshot completes in <50ms when capture buffer is pre-warmed |
| E2.3 | Input injection latency | Key/click events complete in <10ms |
| E2.4 | JSON-RPC overhead | Request dispatch adds <1ms overhead |
| E2.5 | Memory footprint | Binary <10MB, runtime <50MB including buffers |

### E3 — Security and Permissions

| Story | Description | Acceptance Criteria |
|-------|-------------|---------------------|
| E3.1 | macOS TCC integration | Requests Screen Recording and Accessibility permissions correctly |
| E3.2 | Linux capability management | Supports CAP_SYS_ADMIN via setcap or uinput group membership |
| E3.3 | Windows elevation handling | Detects and reports UIPI restrictions gracefully |
| E3.4 | Input validation | All JSON-RPC parameters validated before processing |
| E3.5 | Permission denied errors | Clear error messages when permissions are insufficient |

### E4 — Language Bindings and Integrations

| Story | Description | Acceptance Criteria |
|-------|-------------|---------------------|
| E4.1 | Python SDK | `pip install bare-cua` works; Python examples in README |
| E4.2 | JavaScript/TypeScript SDK | npm package with TypeScript definitions |
| E4.3 | Go SDK | Go module with idiomatic API |
| E4.4 | C# SDK | NuGet package with .NET 6+ support |

### E5 — Plugin Ecosystem

| Story | Description | Acceptance Criteria |
|-------|-------------|---------------------|
| E5.1 | Plugin trait | `MethodPlugin` trait enables custom method handlers |
| E5.2 | Plugin registry | Registry discovers and routes custom methods |
| E5.3 | Official plugins | OCR, element detection, and accessibility plugins |
| E5.4 | Third-party plugins | Community plugin registry with discovery mechanism |

### E6 — AI Integration

| Story | Description | Acceptance Criteria |
|-------|-------------|---------------------|
| E6.1 | Vision model interface | Screenshot output compatible with GPT-4V, Claude, Gemini APIs |
| E6.2 | Agent examples | Reference implementations for popular agent frameworks |
| E6.3 | Tool-calling schema | OpenAPI schema for LLM tool-calling integration |
| E6.4 | State diff detection | `analysis.diff` identifies UI changes between screenshots |

### E7 — Extensibility

| Story | Description | Acceptance Criteria |
|-------|-------------|---------------------|
| E7.1 | Custom adapters | Port trait enables hardware/VNC/remote adapters |
| E7.2 | Protocol alternatives | Support for MCP (Model Context Protocol) alongside JSON-RPC |
| E7.3 | WASM runtime | Plugin system supports WebAssembly-compiled extensions |
| E7.4 | Browser integration | CDP-based browser automation as optional adapter |

## Non-Goals

- Mobile platform support (iOS, Android) in v1
- Browser-embedded automation (use Playwright/Selenium for web)
- Native voice control
- Built-in LLM integration (bare-cua provides the tools; LLM integration is user choice)

## Success Metrics

| Metric | v1 Target | v2 Target |
|--------|-----------|-----------|
| Supported platforms | 3 (Windows, macOS, Linux) | 3 + BSD |
| Capture latency (warm) | <50ms | <20ms |
| Input latency | <10ms | <5ms |
| Binary size | <10MB | <5MB |
| SDK bindings | Python, JS, Go | +C#, +Ruby, +Rust |
| Plugin count | 5 official | 50 community |
| CI test coverage | 80% | 95% |

## Roadmap

### v1.0 (Current)
- Core capture/input/window/process ports
- JSON-RPC 2.0 over stdio
- Platform adapters: WGC, CoreGraphics, X11
- Cross-platform fallback: xcap, enigo
- Python and JavaScript SDKs

### v1.1 (Next Quarter)
- Wayland support via PipeWire portal
- Plugin system (official release)
- OCR plugin (Tesseract integration)
- Browser plugin (CDP-based)

### v2.0 (12 Months)
- AI-native architecture with built-in VLM support
- WASM plugin runtime
- Distributed agent support (multi-machine)
- Enterprise features (audit logging, RBAC)

## Open Questions

1. **Wayland priority**: Should Wayland support precede or follow browser integration?
2. **MCP compatibility**: Should we adopt MCP as primary protocol, keeping JSON-RPC as fallback?
3. **Pricing model**: Free for open source; commercial support/licensing for enterprise?
4. **Cloud deployment**: Support headless cloud instances with virtual display (Xvfb)?

## Appendix: Related Documents

- [SPEC.md](./SPEC.md) — Technical specification and architecture
- [ADR.md](./ADR.md) — Architecture decision records
- [docs/adr/](./docs/adr/) — Detailed ADRs for platform, plugin, and security decisions
- [docs/research/SOTA-2026.md](./docs/research/SOTA-2026.md) — State-of-the-art analysis
