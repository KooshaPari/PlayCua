# bare-cua Deprecation

Phase 1 of the PlayCua + bare-cua merge makes PlayCua the canonical home for
the native computer-use automation crate.

- The standalone `KooshaPari/bare-cua` GitHub name was folded into
  **`KooshaPari/PlayCua`**. Visiting `https://github.com/KooshaPari/bare-cua`
  redirects here; there is no separate live archive repo under that name.
- Historical freeze point for the pre-rename tree: 2026-06-08 snapshot.
- The merged Rust crate is `playcua-native` at version `0.1.0` (**not** on
  crates.io yet — install from this git repo; see README Install).
- The canonical workspace is this repository root with `native` as the Rust
  workspace member (plus local `crates/pheno-*` support crates).
- The CLI and MCP binaries are `playcua-cli` and `playcua-mcp`.
- Python bindings live under `python/` as `playcua` (**not** on PyPI yet —
  `pip install -e ./python` from a clone).
- Do not open new work against a separate `bare-cua` tree; use PlayCua.

Historical consumers should migrate from:

- `bare-cua-native` to `playcua-native`
- `bare-cua-cli` to `playcua-cli`
- `bare-cua-mcp` to `playcua-mcp`
- `bare_cua_native` to `playcua_native`
- `bare_cua` to `playcua`
