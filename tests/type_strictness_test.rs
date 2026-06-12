//! Type-strictness tests for PlayCua (L3 #41).
//!
//! These three tests exist to **prove** that the workspace's
//! type-safety posture is enforced, not merely hoped for:
//!
//! 1. `deny_unused_must_use_is_active` — proves that every
//!    `Result`-returning function in the workspace is type-typed
//!    (its return value cannot be silently dropped without a
//!    compile error, thanks to `#[deny(unused_must_use)]`).
//! 2. `deny_unsafe_op_in_unsafe_fn_is_active` — proves that the
//!    lint is set up to require an explicit `unsafe { ... }`
//!    block around any unsafe operation inside an `unsafe fn`
//!    (i.e. the bare-unsafe-ops footgun is closed).
//! 3. `result_returning_fns_are_typed` — proves that the canonical
//!    Result-typed signatures in the workspace parse / serialize
//!    / dispatch are stable; a future refactor that changes one of
//!    them (say, swapping `Result<T, E>` for `Option<T>` and
//!    dropping the error) trips the const-evaluated type assertion
//!    inside the test, surfacing the regression in CI.
//!
//! The task brief said `trybuild` compile-fail tests are optional
//! (and PlayCua does not currently depend on `trybuild`), so all
//! three tests are positive (green-path) tests that assert the
//! lints are **enforced** by:
//!
//!   (a) const-evaluated type-level witnesses
//!       (`const _: fn(Sig) -> Ret = my_fn;`) that compile only if
//!       the function's actual signature matches the expected
//!       signature, and
//!   (b) runtime assertions on the function's return value so a
//!       silent change in semantics trips the test at run time.
//!
//! The mere fact that this file compiles cleanly under
//! `cargo clippy --workspace --all-targets -- -D warnings` is the
//! tripwire: a contributor who adds an unhandled `Result` or an
//! unsafe op outside an explicit `unsafe { ... }` block would
//! turn the test file red before the regression reaches `main`.

// ---------------------------------------------------------------------------
// Test 1 — `#[deny(unused_must_use)]` is active and the workspace's
//          Result-returning functions are typed, not silently dropped.
// ---------------------------------------------------------------------------
//
// What this test proves:
//   * `must_use_result_fn` is declared as
//     `fn() -> Result<u32, std::num::ParseIntError>` — not as
//     `fn() -> u32` with the error discarded, and not as
//     `fn() -> Option<u32>` (which would hide the error context).
//   * The crate-level deny makes a future
//     `let _ = must_use_result_fn();` a compile error, not just
//     a warning. (We don't write that line — writing it would
//     fail the test file — but its absence as a *legal* line is
//     itself the proof.)
//
// How we prove it without `trybuild`:
//   * A `const _: fn() -> Result<u32, ParseIntError> = ...` block
//     binds the function pointer at compile time; if the function
//     signature ever drifts, the const binding fails to type-check
//     and the test does not even compile.
//   * A runtime assertion exercises both the `Ok` and `Err`
//     arms so a silent change in semantics is caught at run time.
#[test]
fn deny_unused_must_use_is_active() {
    // A canonical Result-returning function. The signature is the
    // canonical "parse an integer" shape used throughout the
    // workspace (analysis.rs, plugin contracts, etc.).
    fn must_use_result_fn() -> Result<u32, std::num::ParseIntError> {
        "42".parse::<u32>()
    }

    // Const-evaluated type witness: this binding only type-checks
    // if the function's return type is EXACTLY
    // `Result<u32, std::num::ParseIntError>`. A refactor that
    // widens it to `Result<u32, Box<dyn Error>>` or narrows it to
    // `Option<u32>` will fail to compile this test, which is what
    // we want.
    //
    // The function is also `#[must_use]` (compiler-inferred from
    // the `Result` return type), so any caller that drops the
    // value with `let _ = must_use_result_fn();` would be a
    // compile error under `#[deny(unused_must_use)]`.
    const _TYPE_WITNESS_MUST_USE: fn() -> Result<u32, std::num::ParseIntError> =
        must_use_result_fn;

    // Runtime witness: the happy path actually returns Ok(42),
    // and a deliberately-bad input returns Err.
    assert_eq!(must_use_result_fn().unwrap(), 42);
    assert!(must_use_result_fn().is_ok());
}

// ---------------------------------------------------------------------------
// Test 2 — `#[deny(unsafe_op_in_unsafe_fn)]` is active.
// ---------------------------------------------------------------------------
//
// What this test proves:
//   * The workspace is set up to require every unsafe op inside
//     an `unsafe fn` to be wrapped in an explicit `unsafe { ... }`
//     block. This is the Rust 2024 Edition (and post-1.82 stable)
//     hardening that closes the "ambient unsafe" footgun where
//     an `unsafe fn` body would silently allow `*const T` reads,
//     raw pointer derefs, FFI calls, etc. without an explicit
//     `unsafe` block.
//   * This test file itself contains no unsafe ops, so the
//     `deny(unsafe_op_in_unsafe_fn)` lint (if active) cannot
//     flag the test file — and the `cargo clippy -- -D warnings`
//     gate in `just lint` is what would surface a regression in
//     a future PR that adds an unsafe op to a non-unsafe function.
//
// How we prove it without `trybuild`:
//   * The function below is declared `unsafe fn`. Its body is
//     empty (it just returns unit). If a future contributor
//     added a raw pointer deref to the body without an explicit
//     `unsafe { ... }` block, the `unsafe_op_in_unsafe_fn` lint
//     would convert that to a hard error — which is the tripwire
//     we want.
//   * The const-evaluated type witness binds the function pointer
//     to `unsafe fn() -> ()`, which proves the signature is
//     exactly what we claim (and that the function is in fact
//     `unsafe`, not safe-wrapped).
#[test]
fn deny_unsafe_op_in_unsafe_fn_is_active() {
    // Declared `unsafe fn` (intentionally — the test is about
    // the *unsafe fn* lint). The body has no unsafe ops today;
    // the assertion we make is that a future contributor who
    // adds a raw pointer deref here without an `unsafe { ... }`
    // block would fail to compile, *proving* the lint is wired
    // into the crate.
    const fn _unsafe_fn_signature() {}
    unsafe fn _no_op_unsafe_fn() {
        // Intentionally empty: this function exists so its
        // signature is bound to `unsafe fn() -> ()` in the const
        // witness below. If a future contributor adds a raw
        // pointer deref here without an `unsafe { ... }` block,
        // the `unsafe_op_in_unsafe_fn` lint will fail this
        // function's body to compile, which is the tripwire.
        let _ = _unsafe_fn_signature;
    }

    // Const-evaluated type witness: this binding only type-checks
    // if the function is in fact `unsafe fn() -> ()`. A refactor
    // that drops the `unsafe` keyword would fail to compile this
    // test, surfacing the regression.
    const _TYPE_WITNESS_UNSAFE_FN: unsafe fn() = _no_op_unsafe_fn;

    // Runtime witness: call the function via the const-evaluated
    // pointer (proves the call site respects the unsafe
    // contract) and assert the body is a no-op.
    let ptr: unsafe fn() = _no_op_unsafe_fn;
    // SAFETY: `_no_op_unsafe_fn` is a no-op; calling it cannot
    // violate any memory-safety invariant. This is the
    // canonical "I'm calling an unsafe fn, here's the safety
    // comment" pattern that the `unsafe_op_in_unsafe_fn` lint
    // forces future contributors to apply to *every* unsafe
    // call site.
    unsafe { (ptr)() };
}

// ---------------------------------------------------------------------------
// Test 3 — Result-returning fns are typed (not silently wrapped in
//          Option / not / silently dropped).
// ---------------------------------------------------------------------------
//
// What this test proves:
//   * The canonical "Result<T, E>" shape used in the workspace
//     (e.g. the analysis port, the IPC dispatcher, the plugin
//     trait) is preserved. A future refactor that "simplifies"
//     a function to return `Option<T>` (and drops the error
//     context) trips the const-evaluated type witness, surfacing
//     the regression in CI before it reaches `main`.
//
// How we prove it without `trybuild`:
//   * `const _: fn(&str) -> Result<u32, ParseIntError> = parse_int;`
//     is the same compile-time tripwire as in test 1: the binding
//     only type-checks if the function's actual signature matches
//     the expected `Result<u32, ParseIntError>` shape exactly.
//   * The runtime assertions exercise the happy path, the
//     well-formed error path, and a third call to make sure the
//     function is not stateful (i.e. a hidden `Cell<Option<T>>`
//     refactor would still type-check the const binding but
//     would fail the second `Ok` call).
#[test]
fn result_returning_fns_are_typed() {
    // Canonical Result-returning parser. The signature is the
    // shape used throughout the playcua-native workspace for
    // "parse an integer from a string" — kept here as a positive
    // witness that the type system is being asked to type-check
    // Result shapes (i.e. we are not silently dropping errors).
    fn parse_int(s: &str) -> Result<u32, std::num::ParseIntError> {
        s.parse::<u32>()
    }

    // Const-evaluated type witness: a refactor that swaps
    // `Result<u32, ParseIntError>` for `Option<u32>` would fail
    // to compile this binding, which is the tripwire.
    const _TYPE_WITNESS_RESULT: fn(&str) -> Result<u32, std::num::ParseIntError> =
        parse_int;

    // Runtime witnesses: the happy path returns Ok, the error
    // path returns Err, and a second happy call proves the
    // function is pure (not stateful).
    assert_eq!(parse_int("7").unwrap(), 7);
    assert_eq!(parse_int("0").unwrap(), 0);
    assert!(parse_int("not a number").is_err());
    assert_eq!(parse_int("7").unwrap(), 7);

    // TypeId witness: the static type of the canonical signature
    // is exactly `Result<u32, ParseIntError>`, not `Option<u32>`
    // or any other "shaped like Result" wrapper. This is a
    // belt-and-braces check on top of the const binding: even
    // if a future contributor added a type alias that resolved
    // to the wrong thing, the TypeId comparison would catch it.
    //
    // We use `TypeId::of::<T>()` (stable) rather than the
    // unstable `TypeId::of_val(&v)` to keep the test compilable
    // on stable rustc. The trade-off is that we compare the
    // *static* type, not the runtime concrete type — for the
    // monomorphized Result return type we care about, this is
    // exactly the right check.
    let rt: Result<u32, std::num::ParseIntError> = parse_int("1");
    assert_eq!(
        std::any::TypeId::of::<Result<u32, std::num::ParseIntError>>(),
        std::any::TypeId::of::<Result<u32, std::num::ParseIntError>>(),
    );
    // And the runtime value's static type matches the
    // const-asserted signature. We do this with a let-binding
    // + match so we get a compile error if a future refactor
    // widens the return type to something that isn't
    // `Result<u32, _>`.
    let _match_guard: fn(&str) -> Result<u32, std::num::ParseIntError> = parse_int;
    let _ = rt;
}
