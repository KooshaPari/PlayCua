# Computer-Use Framework Deduplication Audit

**Date:** 2026-06-14
**Scope:** PlayCua (C:/Users/koosh/Dev/PlayCua) vs Eidolon (E:/Dev/Eidolon-worker)
**Author:** Read-only audit; no code changes
**Status:** Phenotype-org redundancy cluster

---

## 1. Executive Summary

Both projects are Phenotype-org computer-use / device-automation frameworks written in Rust. They share the same organizational context but have **non-overlapping scopes** and **different architectural philosophies**. The core question is not whether to merge them into one codebase, but whether Eidolon should **consume** PlayCua as its desktop implementation, or whether both should remain independent with a shared trait boundary.

**Recommendation:** Differentiation with a unidirectional dependency — Eidolon-desktop should adopt PlayCua's adapter implementations behind the `DesktopAutomator` trait. PlayCua remains the "native stdio binary" product; Eidolon remains the "unified trait-based collection" product. Do not merge repos. Do not duplicate adapter code.

---

## 2. Project Inventory

### 2.1 PlayCua (bare-cua)

| Metric | Value |
|--------|-------|
| Total files | 171 |
| Total lines | 15,509 |
| Rust files (`*.rs`) | 43 (3,889 lines) |
| Python files (`*.py`) | 8 (1,543 lines) |
| C# files (`*.cs`) | 1 (382 lines) |
| Contracts / JSON | 1 OpenRPC spec (427 lines) |
| Workspace members | 1 (`native`) |
| License | MIT |

**Key source files:**
- `native/src/main.rs:1` — stdio JSON-RPC 2.0 server loop
- `native/src/ipc/dispatcher.rs:18` — 14-method JSON-RPC dispatcher
- `native/src/app/mod.rs:18` — DI wiring; platform adapter selection via `cfg(target_os)`
- `native/src/ports/mod.rs:17` — 5 port traits: `CapturePort`, `InputPort`, `WindowPort`, `ProcessPort`, `AnalysisPort`
- `native/src/adapters/` — concrete implementations (xcap, enigo, WGC, SendInput, X11, uinput, CGEvent, etc.)
- `native/src/plugins/mod.rs:12` — `MethodPlugin` trait + `PluginRegistry`
- `python/bare_cua/computer.py:28` — async Python client over stdio subprocess
- `python/bare_cua/agent.py:44` — Anthropic-driven screenshot-action loop agent
- `bindings/csharp/BareCua.cs:22` — C# `NativeComputer` client (IAsyncDisposable)
- `contracts/openrpc.json:1` — OpenRPC 1.2.6 spec (14 methods)

**Architecture:** Hexagonal (ports & adapters) + JSON-RPC 2.0 over stdio + subprocess-based polyglot bindings. No VM, no Docker, no network socket.

**Platform coverage:** Windows (WGC, SendInput, EnumWindows), Linux (X11, uinput, EWMH), macOS (CoreGraphics, CGEvent, NSWorkspace), plus cross-platform fallbacks (xcap, enigo).

### 2.2 Eidolon

| Metric | Value |
|--------|-------|
| Total files | 216 |
| Total lines | 6,025 |
| Rust files (`*.rs`) | 16 (1,270 lines) |
| Python files (`*.py`) | 2 (7 lines) |
| Workspace members | 4 (`eidolon-core`, `eidolon-desktop`, `eidolon-mobile`, `eidolon-sandbox`) |
| License | MIT OR Apache-2.0 |

**Key source files:**
- `crates/eidolon-core/src/traits.rs:6` — 3 traits: `DesktopAutomator`, `MobileAutomator`, `SandboxAutomator`
- `crates/eidolon-core/src/event.rs:8` — `AutomationEvent` with UUID + timestamp + payload
- `crates/eidolon-core/src/input.rs:5` — `PointerInput`, `TextInput` structs
- `crates/eidolon-core/src/viewport.rs:5` — `Viewport` with DPI + orientation
- `crates/eidolon-core/src/error.rs:4` — re-exports `phenotype_errors::{PhenoError, Result}`
- `crates/eidolon-desktop/src/lib.rs:9` — `DesktopClient` stub (all methods log + return Ok)
- `crates/eidolon-mobile/src/lib.rs:16` — `MobileClient` stub (all methods log + return Ok)
- `crates/eidolon-sandbox/src/lib.rs:16` — `SandboxClient` stub (all methods log + return Ok)
- `crates/eidolon-sandbox/src/docker/mod.rs:9` — `DockerOrchestrator` trait (placeholder)
- `crates/eidolon-mobile/src/native/mod.rs:8` — `IosTestAdapter`, `AndroidTestAdapter` traits (placeholder)
- `docs/ADR-001-trait-based-core.md:1` — architectural decision record (trait-based core vs. direct merge)
- `docs/EXTRACTION_PLAN.md:1` — phased extraction plan from KDesktopVirt, kmobile, KVirtualStage, PlayCua, bare-cua
- `release-registry.toml:1` — release metadata; all 4 crates at `0.0.1`, status = `stub` or `alpha`

**Architecture:** Trait-based core with zero inter-crate dependencies; each implementation consumes only `eidolon-core` traits. All implementations are currently stubs.

**Platform coverage:** Planned — desktop (macOS, Windows, Linux), mobile (iOS, Android), sandbox (Docker, nanoVMs, KVM). No working native implementations yet.

---

## 3. Scope Comparison

### 3.1 Overlap Matrix

| Capability | PlayCua | Eidolon | Overlap |
|-----------|---------|---------|---------|
| Desktop screenshot (native) | Full (WGC, CG, X11, xcap) | Stub (plans FFmpeg) | High — same domain, different tech |
| Desktop input injection | Full (SendInput, CGEvent, uinput, enigo) | Stub (no impl) | High — same domain, no code |
| Window enumeration / focus | Full (EnumWindows, EWMH, NSWorkspace) | None | None |
| Process launch / kill | Full (`std::process` + platform kill) | None | None |
| Image diff / hash | Full (BLAKE3, pixel-level diff) | None | None |
| Mobile automation | None | Stub (XCTest/UiAutomator planned) | None |
| Sandbox / container automation | None | Stub (Docker/nanoVMs planned) | None |
| Audit event log | None (tracing JSON to stderr) | Full (`AutomationEvent` + UUID) | Partial — different goals |
| JSON-RPC 2.0 stdio protocol | Full (14 methods, OpenRPC spec) | None | None |
| Polyglot bindings (Python, C#) | Full | None | None |
| Plugin system | Full (`MethodPlugin` registry) | None | None |
| Trait-based abstraction | None (ports are async traits, but not exposed as a library) | Full (`DesktopAutomator`, etc.) | Partial — PlayCua ports are internal |
| Cross-project extraction plan | None | Full (ADR-001 + EXTRACTION_PLAN) | None |

### 3.2 Key Observations

1. **PlayCua is a product; Eidolon is a collection.** PlayCua ships a working binary + polyglot clients. Eidolon is a trait specification with four stub crates waiting for implementations.

2. **Eidolon already names PlayCua as a source.** `docs/EXTRACTION_PLAN.md:43` explicitly lists PlayCua under "Source Projects" and plans to extract its "Virtual display manager" and "Display resolution/DPI config" into `eidolon-sandbox` and `eidolon-core::Viewport` respectively.

3. **PlayCua's `ports/` are essentially `DesktopAutomator` decomposed.** The 5 port traits (`CapturePort`, `InputPort`, `WindowPort`, `ProcessPort`, `AnalysisPort`) map to the 5 methods on `DesktopAutomator` (screenshot, pointer, text, viewport, plus extras). The difference is granularity: PlayCua splits screenshot + input + window + process + analysis into separate traits; Eidolon collapses them into one trait.

4. **Eidolon-desktop is 100% stub.** `crates/eidolon-desktop/src/lib.rs:27` — every method is `// TODO: Integrate native APIs` and returns `Ok(...)` with a log. PlayCua has 3,889 lines of working Rust adapter code across 43 files.

5. **No shared dependencies.** PlayCua uses `anyhow`, `tracing`, `enigo`, `xcap`, `image`, `blake3`, `windows-rs`, `x11rb`. Eidolon uses `log`, `phenotype-errors`, `phenotype-bus`, `uuid`. Different logging, error handling, and async ecosystems.

---

## 4. Architecture Comparison

### 4.1 PlayCua — Hexagonal + Subprocess

```
┌─────────────────────────────────────────────────────────────┐
│  Python / C# / Any JSON-RPC client                          │
│    Computer (subprocess)                                    │
├─────────────────────────────────────────────────────────────┤
│  bare-cua-native (Rust binary)                               │
│    main.rs → ipc/dispatcher.rs → ports/ → adapters/          │
│    JSON-RPC 2.0 over stdio                                   │
└─────────────────────────────────────────────────────────────┘
```

**Design decisions:**
- `native/src/app/mod.rs:20` — Compile-time adapter selection via `cfg(target_os)`
- `native/src/ports/mod.rs:17` — 5 fine-grained async traits (internal boundary)
- `native/src/plugins/mod.rs:12` — Plugin trait for extensible method dispatch
- `contracts/openrpc.json:1` — Contract-first (OpenRPC 1.2.6) before code
- `native/src/main.rs:21` — Single binary, no library distribution needed

**Strengths:**
- Works today (real adapters for all 3 desktop platforms)
- Polyglot bindings are trivial (any language can spawn a subprocess and write JSON)
- No network attack surface (stdio only)
- Plugin system allows third-party extensions without core changes

**Weaknesses:**
- Not usable as a Rust library (binary-first; `lib.rs` exists but is secondary)
- No mobile or sandbox coverage
- No unified audit/event trail (`tracing` JSON logs are unstructured)
- No cross-project trait boundary (ports are internal to the binary)

### 4.2 Eidolon — Trait-Based Collection

```
┌─────────────────────────────────────────────────────────────┐
│  eidolon-core (traits, events, errors, viewport)             │
├─────────────────────────────────────────────────────────────┤
│  eidolon-desktop │ eidolon-mobile │ eidolon-sandbox          │
│  (stubs)         │ (stubs)        │ (stubs)                  │
└─────────────────────────────────────────────────────────────┘
```

**Design decisions:**
- `crates/eidolon-core/src/traits.rs:6` — 3 coarse-grained async traits
- `crates/eidolon-core/src/event.rs:8` — Unified `AutomationEvent` with UUID + timestamp
- `crates/eidolon-core/src/viewport.rs:5` — `Viewport` abstraction (desktop + mobile)
- `docs/ADR-001-trait-based-core.md:7` — Explicitly rejected direct code merge; chose trait-based core
- `docs/EXTRACTION_PLAN.md:59` — Phased extraction from 5 sibling projects

**Strengths:**
- Clear abstraction boundary (`eidolon-core` is a pure interface crate)
- Supports desktop, mobile, and sandbox in one conceptual model
- Audit trail (`AutomationEvent`) is first-class, not a log side-effect
- Incremental extraction plan (no big-bang migration)

**Weaknesses:**
- Zero working implementations (all 4 crates are stubs)
- No polyglot bindings (Rust-only API)
- No IPC protocol (in-process trait calls only)
- Extraction plan is aspirational; no code has been moved yet

---

## 5. Deduplication Analysis

### 5.1 What is Actually Duplicated?

**Nothing is currently duplicated in code.** The two projects do not share source files, crates, or dependencies. The "duplication" is at the **organizational level**: two Rust projects in the same org that both deal with desktop automation.

**Potential future duplication (risk):**
- If Eidolon-desktop implements its own WGC/SendInput/X11 adapters from scratch, it will duplicate 3,889 lines of working PlayCua code.
- If PlayCua adds mobile/sandbox support, it will duplicate Eidolon's planned scope.
- If both projects keep their own error types, they will duplicate `thiserror` enum definitions.

### 5.2 What is Complementary?

| PlayCua provides | Eidolon provides | Combined value |
|------------------|------------------|----------------|
| Working desktop adapters | Trait abstraction for all 3 platforms | Eidolon-desktop can wrap PlayCua's binary or ports |
| JSON-RPC 2.0 stdio protocol | In-process Rust trait API | Polyglot clients can use stdio; Rust apps can use traits |
| Polyglot bindings (Python, C#) | Mobile + sandbox stubs | Eidolon gets instant Python/C# clients via PlayCua's protocol |
| Plugin system | Audit event trail | Plugins can emit `AutomationEvent` into Eidolon's log |
| Cross-platform screenshot pipeline | Viewport + DPI abstraction | Unified `Viewport` across all platforms |

---

## 6. Recommendations

### 6.1 Primary Recommendation: Differentiation with Unidirectional Dependency

**Do not merge the repos.** PlayCua and Eidolon serve different consumers:

- **PlayCua** = "I want a working desktop automation binary I can call from Python/C#/."
- **Eidolon** = "I want a Rust trait library that abstracts desktop, mobile, and sandbox automation."

**Instead, establish a dependency:**

1. **Eidolon-desktop should adopt PlayCua's adapter implementations.**
   - Option A: Spawn the `bare-cua-native` binary as a subprocess and implement `DesktopAutomator` by translating trait calls to JSON-RPC requests (same pattern as Python/C# bindings).
   - Option B: Import `bare_cua_native` as a library crate and wrap its `ports/` behind the `DesktopAutomator` trait.
   - Option C: Extract PlayCua's `ports/` + `adapters/` into a standalone crate (e.g., `playcua-desktop-core`) that both PlayCua and Eidolon depend on.

2. **PlayCua should remain the binary product.**
   - Keep the stdio JSON-RPC protocol as the primary interface.
   - Keep the polyglot bindings in `python/` and `bindings/csharp/`.
   - Do not add mobile or sandbox scope. That is Eidolon's domain.

3. **Shared infrastructure (minimal, crate-level):**
   - **Error types:** Eidolon uses `phenotype-errors` (external). PlayCua uses `thiserror` (internal). Recommendation: PlayCua should migrate to `phenotype-errors` so both projects share the same error taxonomy. This is a 1–2 day refactor.
   - **Viewport:** Eidolon's `Viewport` (`crates/eidolon-core/src/viewport.rs:5`) is more mature (DPI, orientation). PlayCua has no equivalent domain type. Recommendation: PlayCua should adopt `eidolon-core::Viewport` as its display metadata type if/when it needs structured viewport info.
   - **Event trail:** PlayCua's `tracing` JSON logs are unstructured. Eidolon's `AutomationEvent` (`crates/eidolon-core/src/event.rs:8`) is structured. Recommendation: PlayCua should optionally emit `AutomationEvent` into a channel that Eidolon can consume, but this is a future enhancement, not a blocker.

### 6.2 Secondary Recommendation: Do Not Let Eidolon Reimplement Desktop Adapters

**Cost of duplication:** If Eidolon-desktop writes its own WGC/SendInput/X11/CG adapters from scratch, the org will maintain ~3,500 lines of redundant Rust code across two repos. This violates the Phenotype-org "Cross-Project Reuse" mandate (`CLAUDE.md:47` — "proactively identify code that is sharable").

**Cost of dependency:** If Eidolon-desktop shells out to `bare-cua-native`, the dependency is a single binary artifact. If Eidolon-desktop imports `bare_cua_native` as a library, the dependency is a single crate in the workspace. Both are lower cost than rewriting.

### 6.3 Tertiary Recommendation: Shared Crate Extraction (Future)

If both projects need to share more than just the binary interface, consider extracting a `phenotype-desktop-core` crate containing:
- `CapturePort` + `InputPort` + `WindowPort` + `ProcessPort` + `AnalysisPort` (from PlayCua)
- `DesktopAutomator` (from Eidolon, as a composite trait)
- Platform adapters (WGC, SendInput, X11, uinput, CG, etc.)
- `Viewport` (from Eidolon)
- `AutomationEvent` (from Eidolon)

This crate would be a library only (no JSON-RPC, no main function). PlayCua would depend on it and add its JSON-RPC dispatcher + stdio loop. Eidolon would depend on it and add its mobile/sandbox traits + audit layer.

**Effort estimate:** 2–3 weeks to extract, test, and wire into both repos. Not urgent; can be deferred until Eidolon-desktop is ready to move beyond stubs.

---

## 7. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Eidolon reimplements desktop adapters from scratch | Medium | High (3,500 lines of duplicated code) | Explicitly assign PlayCua as the desktop adapter owner in Eidolon's EXTRACTION_PLAN |
| PlayCua expands into mobile/sandbox, duplicating Eidolon | Low | Medium | Reject PRs that add mobile/sandbox scope to PlayCua; redirect to Eidolon |
| Divergent error types make integration painful | Medium | Low | Migrate PlayCua to `phenotype-errors` (small refactor) |
| PlayCua's binary-first design makes library reuse hard | Medium | Medium | Add `lib.rs` feature parity or extract `native/` into a library crate |
| Eidolon's stub status means dependency never materializes | High | Low | Set milestone: Eidolon-desktop must use PlayCua adapters before writing new ones |

---

## 8. Action Items

| # | Action | Owner | Priority |
|---|--------|-------|----------|
| 1 | Update `Eidolon/docs/EXTRACTION_PLAN.md` to name PlayCua as the desktop adapter source (not just "Virtual display manager" and "DPI config") | Eidolon maintainers | High |
| 2 | Add a `DesktopAutomator` implementation in `eidolon-desktop` that wraps `bare-cua-native` via subprocess JSON-RPC | Eidolon maintainers | High |
| 3 | Migrate PlayCua's error types from `thiserror` internal enums to `phenotype-errors` crate | PlayCua maintainers | Medium |
| 4 | Reject any PlayCua PRs that add mobile or sandbox scope | PlayCua maintainers | Medium |
| 5 | Create a shared `phenotype-desktop-core` crate (future) if both projects need in-process Rust library access | Phenotype-org | Low |
| 6 | Add `AutomationEvent` emission channel to PlayCua's plugin system (future) | PlayCua maintainers | Low |

---

## 9. File References (Real Counts)

### PlayCua
- `native/src/main.rs:1` (76 lines) — binary entry point
- `native/src/ipc/dispatcher.rs:18` (372 lines) — 14-method JSON-RPC dispatcher
- `native/src/app/mod.rs:18` (145 lines) — DI wiring, platform adapter selection
- `native/src/ports/mod.rs:17` (64 lines) — 5 port traits
- `native/src/adapters/` (12+ files) — concrete platform adapters
- `native/src/domain/` (5 modules) — pure business types (Frame, Key, MouseEvent, WindowInfo, etc.)
- `native/src/plugins/mod.rs:12` (107 lines) — plugin trait + registry
- `python/bare_cua/computer.py:28` (246 lines) — Python async client
- `python/bare_cua/agent.py:44` (210 lines) — Anthropic agent loop
- `bindings/csharp/BareCua.cs:22` (382 lines) — C# client
- `contracts/openrpc.json:1` (427 lines) — OpenRPC 1.2.6 spec
- `native/tests/unit/analysis_tests.rs:1` (148 lines) — unit tests
- `python/tests/test_computer.py:1` (154 lines) — Python mock-subprocess tests
- `tests/smoke_test.rs:1` (7 lines) — trivial smoke test

### Eidolon
- `crates/eidolon-core/src/traits.rs:6` (85 lines) — 3 automation traits
- `crates/eidolon-core/src/event.rs:8` (65 lines) — `AutomationEvent` + `EventPayload`
- `crates/eidolon-core/src/viewport.rs:5` (37 lines) — `Viewport` with DPI + orientation
- `crates/eidolon-core/src/input.rs:5` (53 lines) — `PointerInput` + `TextInput`
- `crates/eidolon-core/src/error.rs:4` (7 lines) — re-export of `phenotype-errors`
- `crates/eidolon-desktop/src/lib.rs:9` (49 lines) — `DesktopClient` stub
- `crates/eidolon-mobile/src/lib.rs:16` (58 lines) — `MobileClient` stub
- `crates/eidolon-sandbox/src/lib.rs:16` (62 lines) — `SandboxClient` stub
- `crates/eidolon-sandbox/src/docker/mod.rs:9` (44 lines) — `DockerOrchestrator` trait placeholder
- `crates/eidolon-mobile/src/native/mod.rs:8` (26 lines) — iOS/Android adapter trait placeholders
- `docs/ADR-001-trait-based-core.md:1` (96 lines) — architectural decision record
- `docs/EXTRACTION_PLAN.md:1` (93 lines) — phased extraction from 5 sibling projects
- `release-registry.toml:1` (39 lines) — release metadata (all crates `0.0.1`, stub/alpha)

---

## 10. Conclusion

PlayCua and Eidolon are **not redundant codebases** — they are **complementary products** with a potential for clean dependency. PlayCua is the working desktop implementation; Eidolon is the trait-based abstraction layer. The correct path is **differentiation with a unidirectional dependency**: Eidolon consumes PlayCua's desktop adapters, rather than rewriting them. This preserves both projects' identities, avoids duplication, and respects the architectural boundaries each has established.

**Final verdict:** Do not consolidate. Do not merge. Wire Eidolon-desktop to PlayCua. Keep scopes separate.
