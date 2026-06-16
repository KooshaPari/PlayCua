# PlayCua — Requirements Traceability Matrix

> **Phase 3 traceability layer.** This file is the single source of
> truth for "which test proves which FR, and which source line
> implements it". Update this table whenever an FR gains, loses, or
> changes tests.

## Matrix (FR × Source × Test)

| FR    | Implementation anchor                                                                                                          | Test(s)                                                                                                                                                                              | Status   |
|-------|--------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------|
| FR-001 | `native/src/ipc/dispatcher.rs:74,104-131`; `native/src/domain/capture.rs:4-10`; `native/src/ports/capture.rs`                  | `tests/screenshot_test.rs::screenshot_dispatcher_returns_png_envelope` (added Phase 3); `tests/screenshot_test.rs::screenshot_routes_to_capture_port` (added Phase 3)                 | Covered  |
| FR-002 | `native/src/ipc/dispatcher.rs:76-80`; `native/src/domain/input.rs`; `native/src/ports/input.rs`; `native/src/adapters/enigo.rs` | `tests/input_dispatch_test.rs::input_key_click_move_route_via_input_port` (added Phase 3); `tests/input_dispatch_test.rs::input_unknown_method_returns_method_not_found` (added Phase 3) | Covered  |
| FR-003 | `native/src/ipc/dispatcher.rs:82-84`; `native/src/domain/window.rs`; `native/src/ports/window.rs`                              | `native/tests/hexagonal_boundary.rs::dispatcher_routes_windows_methods_via_window_port` (existing; see `native/tests/hexagonal_boundary.rs:1-30` for the architecture claim)        | Covered  |
| FR-004 | `native/src/ipc/dispatcher.rs:86-88`; `native/src/domain/process.rs`; `native/src/ports/process.rs`; `native/src/adapters/process_adapter.rs` | `tests/process_lifecycle_test.rs::process_launch_kill_status_round_trip` (added Phase 3)                                                                                            | Covered  |
| FR-005 | `native/src/ipc/dispatcher.rs:90-91`; `native/src/domain/analysis.rs`; `native/src/ports/analysis.rs`; `native/src/adapters/analysis_adapter.rs` | `native/tests/unit/analysis_tests.rs::diff_identical_frames_returns_zero_change_ratio` (existing); `native/tests/unit/analysis_tests.rs::diff_all_black_vs_all_white_returns_one_change_ratio` (existing); `tests/analysis_hash_test.rs::hash_of_identical_frames_matches` (added Phase 3) | Covered |
| FR-006 | `native/src/ipc/dispatcher.rs:57-72`; `native/src/modality/registry.rs`; `native/src/modality/mod.rs`                          | `native/tests/integration_smoke.rs::playcua_native_library_is_wired` (existing; asserts `modality::ModalityKind::parse("native")` succeeds); `tests/modality_ping_test.rs::ping_reports_modality_kind` (added Phase 3) | Covered |
| FR-007 | `native/src/ipc/dispatcher.rs:19-29`; `native/src/ports/*`; `native/src/adapters/*`                                              | `native/tests/hexagonal_boundary.rs` (existing; full file is the boundary test); `tests/type_strictness_test.rs::deny_unused_must_use_is_active` (existing; type-system tripwire)   | Covered  |

## Test inventory (pre-Phase 3 + Phase 3 delta)

| File                                                            | Pre-Phase 3 `#[test]` count | Phase 3 added | Post-Phase 3 total |
|-----------------------------------------------------------------|-----------------------------|---------------|--------------------|
| `tests/smoke_test.rs`                                           | 1                           | 0             | 1                  |
| `tests/type_strictness_test.rs`                                 | 3                           | 0             | 3                  |
| `native/tests/hexagonal_boundary.rs`                            | (multiple `#[tokio::test]`) | 0             | (existing)         |
| `native/tests/integration_smoke.rs`                             | 6                           | 0             | 6                  |
| `native/tests/integration_screenshot_api.rs`                    | 2                           | 0             | 2                  |
| `native/tests/unit/analysis_tests.rs`                           | (multiple)                  | 0             | (existing)         |
| `crates/port-input/tests/input_test.rs`                         | (multiple)                  | 0             | (existing)         |
| `crates/port-renderer/tests/renderer_test.rs`                   | (multiple)                  | 0             | (existing)         |
| `crates/port-window-mgr/tests/window_mgr_test.rs`               | (multiple)                  | 0             | (existing)         |
| `tests/screenshot_test.rs` (NEW Phase 3)                        | 0                           | 2             | 2                  |
| `tests/input_dispatch_test.rs` (NEW Phase 3)                    | 0                           | 2             | 2                  |
| `tests/process_lifecycle_test.rs` (NEW Phase 3)                 | 0                           | 1             | 1                  |
| `tests/analysis_hash_test.rs` (NEW Phase 3)                     | 0                           | 1             | 1                  |
| `tests/modality_ping_test.rs` (NEW Phase 3)                     | 0                           | 1             | 1                  |
| **Total top-level `tests/` directory `#[test]` functions**      | **4**                       | **7**         | **11**             |

(Phase 3 adds 7 new `#[test]` functions across 5 new files in
`tests/`, bringing the top-level test directory from 4 to 11. The
in-crate integration suites under `native/tests/` and `crates/*/tests/`
are unchanged in count by this phase — they are the "existing
backbone" the new top-level tests complement.)

## FR coverage summary

| FR    | Title                                 | New Phase 3 tests? | Existing test refs                                                         |
|-------|---------------------------------------|--------------------|----------------------------------------------------------------------------|
| FR-001 | Screen Capture                        | Yes (2)            | (none in `tests/`; covered at adapter level in `crates/port-renderer/`)   |
| FR-002 | Input Injection                       | Yes (2)            | (none in `tests/`; covered at port trait level in `crates/port-input/`)   |
| FR-003 | Window Enumeration / Focus / Lookup   | No (0)             | `native/tests/hexagonal_boundary.rs` (full boundary test)                 |
| FR-004 | Process Lifecycle                     | Yes (1)            | (none; previously un-anchored)                                            |
| FR-005 | Image Analysis: Diff and Hash         | Yes (1)            | `native/tests/unit/analysis_tests.rs` (diff path)                          |
| FR-006 | Modality Selection and Self-Reporting | Yes (1)            | `native/tests/integration_smoke.rs::playcua_native_library_is_wired`      |
| FR-007 | Hexagonal Boundary                    | No (0)             | `native/tests/hexagonal_boundary.rs` (entire file) + type-strictness tests |

## Annotation convention

Every `#[test]` function added in Phase 3 begins with a doc-comment
that names the FR it covers, e.g.:

```rust
//! FR-001: Screen Capture — verify the `screenshot` JSON-RPC method
//! returns a `{ data, width, height, format: "png" }` envelope.
```

This is the same convention used in `tests/smoke_test.rs` (which
carries `// Traces to: FR-ORG-AUDIT-2026-04-001`) and in
`native/tests/integration_smoke.rs` (L5 #81 wiring claim references).
The doc comment is greppable:

```sh
rg "FR-00[1-7]" tests/ native/tests/ crates/*/tests/
```

Any new test that lacks an `FR-` reference is a Phase 3 traceability
gap and should be annotated before merge.
