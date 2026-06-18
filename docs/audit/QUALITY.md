# PlayCua Quality Audit

**Date:** 2026-06-14
**Scope:** `C:\Users\koosh\Dev\PlayCua`
**Method:** Read-only, `grep` + source read. No builds, no cargo/go/npm, no git mutations.
**Author:** Forge audit pass

---

## 0. Executive Summary

| Dimension | Headline |
|-----------|----------|
| Test coverage | 17 test fns in 3 test files; **9 of 10** native modules have **zero** tests. Python: 1 file, 6 tests. C#: 0. Sandbox: 0. |
| Debt markers | 7 TODOs, 17 `.unwrap()` in non-test lib code, 1 `.expect()` (in test), 0 `panic!`, 0 `unimplemented!`, 0 `todo!`. |
| Architecture | **~613 LOC of dead code**: top-level `input/`, `window/`, `process/`, `analysis/` are never declared as a module. `ipc/dispatcher.rs` re-implements the same logic. `WindowInfo` duplicated across two type trees with **different `hwnd` types** (`usize` vs `i64`). SPEC.md claims PyO3 but binding is stdio subprocess. |
| LOC | 5,979 source LOC total (3,733 rust + 1,925 py/cs + 321 ps1). No file > 600 lines. Largest is `sandbox/sandboxfile.py` at 425. |

---

## 1. TEST GAPS

### 1.1 Test counts

| Suite | File | Test fns |
|-------|------|---------:|
| Rust integration | `native/tests/unit/analysis_tests.rs:35-148` | **7** |
| Rust unit (inline) | `native/src/plugins/mod.rs:84-106` | **3** |
| Rust smoke | `tests/smoke_test.rs:4-7` | **1** |
| Python pytest | `python/tests/test_computer.py:88-154` | **6** |
| **Total** | **3 test files** | **17** |

The 3 Rust `#[test]`/`#[tokio::test]` markers are at:
`native/tests/unit/analysis_tests.rs:35, 48, 66, 95, 112, 122, 136`
`native/src/plugins/mod.rs:84, 94, 102`
`tests/smoke_test.rs:4`

### 1.2 Modules with ZERO tests

All ten `native/src/` subdirectories, plus `bindings/` and `sandbox/`:

| Module | LOC | Test files | Reason |
|--------|----:|-----------:|--------|
| `native/src/adapters/` | 1,065 | 0 | All platform adapters untested (xcap, enigo, WGC, SendInput, EWMH, uinput, CGEvent, NSWorkspace, CGCapture, X11Capture, process_adapter, analysis_adapter) |
| `native/src/app/` | 145 | 0 | DI wiring (`app/mod.rs:14`) untested |
| `native/src/domain/` | 226 | 0 | All pure types (capture, input, window, process, analysis) untested |
| `native/src/ipc/` | 483 | 0 | `dispatcher.rs` (372 LOC) + `mod_types.rs` (104) — both untested |
| `native/src/ports/` | 64 | 0 | All 5 port traits + their contracts untested |
| `native/src/input/` | 649 | 0 | **Dead code (see §3.1)** — never wired in |
| `native/src/window/` | 259 | 0 | **Dead code** — never wired in |
| `native/src/process/` | 185 | 0 | **Dead code** — never wired in |
| `native/src/analysis/` | 107 | 0 | **Dead code** — never wired in |
| `native/src/plugins/` | 107 | **3 (only module tested)** | Only registry; `MethodPlugin` external impls untested |
| `bindings/csharp/` | 382 | 0 | No test project at all |
| `sandbox/` (py) | 902 | 0 | `config.py`, `sandbox.py`, `sandboxfile.py` — no test file |
| `python/bare_cua/agent.py` | 210 | 0 | Anthropic agent loop untested (only `Computer` client is) |

**Test ratio:** 17 tests / 5,979 LOC ≈ 1 test per 352 LOC. 11 of 13 code units have **0 tests**.

### 1.3 Test-function inventory (file:line)

`native/tests/unit/analysis_tests.rs:36 diff_identical_frames_returns_zero_change_ratio`
`native/tests/unit/analysis_tests.rs:49 diff_all_black_vs_all_white_returns_one_change_ratio`
`native/tests/unit/analysis_tests.rs:67 diff_change_ratio_is_always_in_unit_interval`
`native/tests/unit/analysis_tests.rs:96 diff_dimension_mismatch_returns_error`
`native/tests/unit/analysis_tests.rs:113 hash_same_data_returns_same_string`
`native/tests/unit/analysis_tests.rs:123 hash_different_data_returns_different_strings`
`native/tests/unit/analysis_tests.rs:137 hash_result_is_nonempty_hex_string`
`native/src/plugins/mod.rs:85 test_register_and_find`
`native/src/plugins/mod.rs:95 test_replace_on_duplicate_register`
`native/src/plugins/mod.rs:103 test_find_missing_returns_none`
`tests/smoke_test.rs:5 smoke_test`
`python/tests/test_computer.py:89 test_ping_returns_true`
`python/tests/test_computer.py:100 test_screenshot_returns_bytes`
`python/tests/test_computer.py:108 test_screenshot_starts_with_png_header`
`python/tests/test_computer.py:122 test_unknown_method_raises`
`python/tests/test_computer.py:137 test_context_manager_cleans_up`
`python/tests/test_computer.py:146 test_multiple_calls_in_session`

---

## 2. DEBT

### 2.1 Raw counts (entire repo)

| Marker | Count | In test code? | In lib/main code? |
|--------|------:|:-------------:|:-----------------:|
| `TODO`/`FIXME`/`XXX`/`HACK` | **7** | 0 | 7 |
| `.unwrap()` | **17** | 0 | 17 |
| `.expect(` | **1** | 1 | 0 |
| `panic!(` | **0** | 0 | 0 |
| `unimplemented!(` | **0** | 0 | 0 |
| `todo!(` | **0** | 0 | 0 |
| `raise NotImplementedError` (py) | **0** | 0 | 0 |
| `throw new NotImplementedException` (cs) | **0** | 0 | 0 |

**Zero `panic!` / `unimplemented!` / `todo!` in the entire repo.** This is good.

### 2.2 `unwrap()` in non-test lib code (file:line)

All 17 sites are in non-test production code. Top offenders:

| File | Count | Sites |
|------|------:|-------|
| `native/src/process/mod.rs` | 3 | `:83`, `:119`, `:142` (all `Mutex::lock().unwrap()` on `CHILD_MAP`) |
| `native/src/adapters/process_adapter.rs` | 3 | `:60`, `:74`, `:91` (all `Mutex::lock().unwrap()` on the same global map) |
| `native/src/input/windows.rs` | 2 | `:171` (s.chars().next().unwrap()), `:222` (same) |
| `native/src/main.rs` | 1 | `:28` (env-filter parse — acceptable at startup) |
| `native/src/adapters/windows/wgc.rs` | 1 | `:168` (tx2.lock().unwrap().take() — WGC frame-arrived callback) |
| `native/src/adapters/windows/sendinput.rs` | 1 | `:122` (s.chars().next().unwrap() in key parser) |
| `native/src/adapters/enigo.rs` | 1 | `:158` (s.chars().next().unwrap() in `parse_enigo_key`) |
| `native/src/plugins/mod.rs` | 1 | `:90` (`.unwrap()` on plugin handle — inside `#[tokio::test]`, **acceptable**) |
| `native/src/input/linux.rs` | 1 | `:105` (s.chars().next().unwrap() in key parser) |
| `native/src/input/macos.rs` | 1 | `:103` (s.chars().next().unwrap() in key parser) |

**Flag — 6 unguarded `s.chars().next().unwrap()` calls** across key parsers (enigo.rs:158, sendinput.rs:122, input/{windows,linux,macos}.rs:{171|105|103,222}). All guard with `other if other.len() == 1` so they cannot panic at runtime, but the pattern is duplicated 6× and should be a single helper.

**Flag — 6 `Mutex::lock().unwrap()` on the process child map** (3 in `process/mod.rs`, 3 in `adapters/process_adapter.rs`). Poisoning a `Mutex` is unlikely but recoverable: prefer `.lock().expect("CHILD_MAP poisoned")` or `.expect()`-with-context. These should at minimum use `.lock().expect("process map mutex poisoned")` for a better diagnostic.

### 2.3 `expect(` (1 site)

`native/src/plugins/mod.rs:89` — `registry.find("test.echo").expect("plugin should be registered")` — **inside a `#[tokio::test]`** (line 84). Acceptable for a test.

### 2.4 `TODO` / `FIXME` ranking

| File | Count | Sites |
|------|------:|-------|
| `native/src/window/linux.rs` | 2 | `:2` (module-level), `:29` (focus_window stub) |
| `native/src/window/macos.rs` | 1 | `:28` (focus_window stub) |
| `native/src/adapters/macos/nsworkspace.rs` | 2 | `:4` (module-level), `:74` (focus_window stub) |
| `native/src/adapters/linux/ewmh.rs` | 2 | `:4` (module-level), `:67` (focus_window stub) |

All 7 TODOs are **the same deficiency**: `WindowPort::focus_window` is unimplemented on Linux and macOS. The adapters warn-and-return-`Ok(())` (`ewmh.rs:66-70`, `nsworkspace.rs:73-77`, `window/linux.rs:28-33`, `window/macos.rs:27-31`), so `windows.focus` RPC is a no-op on those platforms. This is a real functional gap, not a stylistic marker.

### 2.5 Bare `assert` in production Python (2 sites, **flag**)

`python/bare_cua/computer.py:105` — `assert self._proc.stdin is not None`
`python/bare_cua/computer.py:108` — `assert self._proc.stdout is not None`

These are **production code** (not under `tests/`). `python -O` strips them. They should be `if x is None: raise RuntimeError(...)`.

### 2.6 Silent `except` (Python) — 2 sites

`sandbox/sandbox.py:59` — `except OSError: pass` (file-drop unlink cleanup)
`sandbox/sandbox.py:138` — `except OSError: pass` (temp-file cleanup at `__aexit__`)

Both are cleanup paths where suppression is intentional. Mild flag only.

---

## 3. ARCHITECTURE (layering & SOLID)

### 3.1 CRITICAL — Dead-code module tree (~613 LOC, 13 files)

The repo contains **two parallel implementations** of the same four concerns. Only the `domain/` + `ports/` + `adapters/` path is wired in; the top-level `input/`, `window/`, `process/`, `analysis/` are **never declared as modules** anywhere in the crate.

Verified by `grep '^use crate::(input|window|process|analysis)'` and `grep '^mod (input|window|process|analysis)'` — **zero matches** outside the directories themselves. `native/src/lib.rs:6-11` declares only `adapters, app, domain, ipc, plugins, ports`; `native/src/main.rs:7-12` mirrors that.

| Dead file | LOC | Status |
|-----------|----:|--------|
| `native/src/input/mod.rs` | 182 | Re-declares `KeyAction/MouseButton/MouseAction/ScrollDirection` (duplicates `domain/input.rs:14-50`) and defines `key_rpc/type_rpc/click_rpc/scroll_rpc/move_rpc` handlers. Never called. |
| `native/src/input/windows.rs` | 235 | Win32 input impl that the dead `input::mod` would have called. |
| `native/src/input/linux.rs` | 117 | Linux enigo impl, dead. |
| `native/src/input/macos.rs` | 115 | macOS enigo impl, dead. |
| `native/src/window/mod.rs` | 94 | Re-declares `WindowInfo` (duplicates `domain/window.rs:5` — see §3.3) and defines `list_rpc/focus_rpc/find_rpc` handlers. Never called. |
| `native/src/window/windows.rs` | 101 | Win32 EnumWindows impl, dead. |
| `native/src/window/linux.rs` | 33 | Linux xcap impl, dead. |
| `native/src/window/macos.rs` | 31 | macOS xcap impl, dead. |
| `native/src/process/mod.rs` | 185 | Re-implements `launch/kill/status` (duplicates `adapters/process_adapter.rs:41-103`) and defines `launch_rpc/kill_rpc/status_rpc`. Never called. |
| `native/src/analysis/mod.rs` | 107 | Re-implements `diff/hash` (duplicates `adapters/analysis_adapter.rs:24-89`) and defines `diff_rpc/hash_rpc`. Never called. |
| `native/src/adapters/{linux,macos,windows}/mod.rs` | 5 each | Empty module forwarders — fine. |
| `native/tests/unit/analysis_tests.rs` | 148 | Tests the **adapters** path, not the dead `analysis` mod. |

**Implication:** `app/mod.rs:14` wires `Arc<dyn {CapturePort,InputPort,WindowPort,ProcessPort,AnalysisPort}>` into the **dispatcher** at `ipc/dispatcher.rs:18`. The dead `*_rpc` functions are never reachable. `ipc/dispatcher.rs:76-357` re-implements what `input/mod.rs:91-123`, `window/mod.rs:31-56`, `process/mod.rs:30-59`, `analysis/mod.rs:13-38` already did — **two parallel RPC layers, only one wired in.**

### 3.2 `WindowInfo` duplication with **type drift**

| Location | `hwnd` type | Source |
|----------|-------------|--------|
| `native/src/domain/window.rs:5-15` | `pub hwnd: usize` | The canonical domain type, used by ports and adapters. |
| `native/src/window/mod.rs:16-25` | `pub hwnd: i64` | The dead-code copy. **Different signedness.** |
| `bindings/csharp/BareCua.cs:356-365` | `long Hwnd` | C# binding deserializes the wire format. |

The wire format is `i64` (the dispatcher emits `hwnd: 0` as a JSON number — see `dispatcher.rs:238-247`). `domain::WindowInfo` declares `usize`, which works on 64-bit only and is a deserialization risk. The adapter chains (`enumwin.rs:93`, `ewmh.rs:36`, `nsworkspace.rs:42`) cast at the boundary.

### 3.3 Misleading adapter name

`native/src/adapters/windows/sendinput.rs` is **named** after the Win32 `SendInput` API, but `SendInputAdapter`'s `InputPort` impl (lines 28-51) just delegates to `EnigoInput`. Only the **free function** `inject_to_hwnd` (line 56) uses `PostMessageW`. The "SendInput" name appears 5 times in this file (`sendinput.rs:1, 14, 28, 32, 45, 54, 75, 88, 128`) but the actual `SendInput` Win32 function is not used. Either rename to `PostMessageInputAdapter` (to match the only real code) or actually use `SendInput` for the system-wide path.

### 3.4 `domain/` purity is intact

Verified: `grep -r 'use crate::' native/src/domain` returns **zero results**. The domain layer has no `xcap`/`enigo`/`windows-rs` imports (`domain/{input,window,process,capture,analysis}.rs` all use only `std`, `serde`, `thiserror`). This is good — the hexagonal boundary is respected on the domain side.

### 3.5 God-dispatcher

`native/src/ipc/dispatcher.rs` is **372 LOC** containing:
- A 14-arm `match` on method names (`dispatcher.rs:43-69`)
- 14 separate `handle_*` methods, each with its own `#[derive(Deserialize)]` param struct
- A private `deserialize_or_default` helper (line 364)
- A `base64` import for the analysis handlers

It is the **only** function-level module that knows all 14 methods. If a 15th method is added, three files change: `ipc/dispatcher.rs`, `ports/mod.rs`, and one adapter. The split between this dispatcher and the dead `*_rpc` handlers is the same split — the file is doing param-deserialization + dispatch + error-translation in one place.

### 3.6 SPEC drift

`SPEC.md:14` claims: `│  │         Python Bindings (PyO3)            │  │`
`SPEC.md:24` says: `| bindings | Python wrapper | \`BARE.execute()\` |`
**Reality:** No PyO3 anywhere. `grep -ri 'pyo3\|maturin\|setuptools-rust'` returns zero. The Python binding is `python/bare_cua/computer.py:28` — an async client that spawns the native binary as a **subprocess over stdio JSON-RPC**. The C# binding is the same pattern (`bindings/csharp/BareCua.cs:22` — `Process.Start` + stdin/stdout). The "PyO3" line is stale documentation.

Also: `python/pyproject.toml:8` description: `"CUA fork - bare metal computer-use agent without VM layer"` — but the repo is `PlayCua`, not `CUA`. Project-name drift.

### 3.7 Adapter delegation chains (DRY)

Several platform adapters are one-line wrappers around `XcapCapture` / `EnigoInput`:

`native/src/adapters/linux/uinput.rs:31-43` — 100% delegates to `EnigoInput`.
`native/src/adapters/macos/cgevent.rs:31-43` — 100% delegates to `EnigoInput`.
`native/src/adapters/linux/x11capture.rs:31-38` — 100% delegates to `XcapCapture`.
`native/src/adapters/macos/cgcapture.rs:31-38` — 100% delegates to `XcapCapture`.

These are 4 thin shells (155 LOC total) that exist **only** to satisfy the `build_input()`/`build_capture()` factory dispatch in `app/mod.rs:84-95, 53-65`. If a future Linux port needs different behavior, these wrappers become meaningful — but today they are pure overhead.

---

## 4. LOC

### 4.1 Totals

| Language / kind | Files | LOC |
|------------------|------:|----:|
| Rust (`native/src/**/*.rs`) | 39 | **3,733** |
| Rust tests (`tests/`, `native/tests/`) | 2 | 155 |
| Python (`python/**`, `sandbox/**`) | 10 | **1,543** |
| C# (`bindings/csharp/`) | 1 | 382 |
| JSON (`contracts/`) | 1 | 427 |
| PowerShell templates (`sandbox/templates/`) | 2 | 321 |
| **Total source LOC** | **55** | **6,561** |

(Test files and contracts/spec are included for transparency; production code is 5,979 LOC.)

### 4.2 Files > 600 lines: **NONE**

Largest files (top 10 by LOC):

| Rank | File | LOC |
|-----:|------|----:|
| 1 | `sandbox/sandboxfile.py` | 425 |
| 2 | `native/src/ipc/dispatcher.rs` | 372 |
| 3 | `native/src/adapters/windows/wgc.rs` | 305 |
| 4 | `bindings/csharp/BareCua.cs` | 382 |
| 5 | `sandbox/config.py` | 258 |
| 6 | `python/bare_cua/computer.py` | 246 |
| 7 | `native/src/input/windows.rs` | 235 |
| 8 | `sandbox/sandbox.py` | 219 |
| 9 | `python/bare_cua/agent.py` | 210 |
| 10 | `native/src/input/mod.rs` (dead) | 182 |

The only file approaching 600 is `sandbox/sandboxfile.py` (425), still well under.

### 4.3 Rust file size distribution (39 files in `native/src/`)

- 0 files in the 400-600 range.
- 3 files in the 300-399 range (`dispatcher.rs` 372, `wgc.rs` 305, plus the dead `input/mod.rs` 182 is in a different bucket).
- Most files (24 of 39, 62%) are under 100 LOC.

---

## 5. Five-line summary

1. **17 tests / 5,979 production LOC** — 9 of 10 native modules, plus all of `bindings/csharp/` and `sandbox/`, have **zero tests**; only `plugins/` and the `analysis_adapter` are exercised.
2. **Zero `panic!`/`unimplemented!`/`todo!`** is good, but **17 unguarded `.unwrap()`** concentrate around the process `Mutex` map (6 sites) and the duplicated enigo key parsers (6 sites).
3. **7 TODOs are one bug** — `WindowPort::focus_window` silently no-ops on Linux and macOS (`adapters/linux/ewmh.rs:66`, `adapters/macos/nsworkspace.rs:73`, `window/{linux,macos}.rs:28-31`).
4. **~613 LOC of dead code** in `native/src/{input,window,process,analysis}/` (13 files) — never declared as modules; `ipc/dispatcher.rs:43-357` is the only wired-in RPC layer and re-implements the dead modules' logic.
5. **Two type drifts** — `WindowInfo.hwnd` is `usize` in `domain/window.rs:5` but `i64` in `window/mod.rs:16` (and `i64` on the wire / C# binding); `SPEC.md:14` claims PyO3 but the binding is stdio subprocess.
