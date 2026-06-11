# bare-cua Deprecation

Phase 1 of the PlayCua + bare-cua merge makes PlayCua the canonical home for
the native computer-use automation crate.

- `bare-cua` is frozen at the 2026-06-08 snapshot.
- The merged Rust crate is `playcua-native` at version `0.1.0`.
- The canonical workspace is this repository root with `native` as the Rust
  workspace member.
- The CLI and MCP binaries are `playcua-cli` and `playcua-mcp`.
- Python bindings are published from `python/` as `playcua`.
- The separate `KooshaPari/bare-cua` repository should receive only archival or
  deprecation-banner updates after this phase.

Historical consumers should migrate from:

- `bare-cua-native` to `playcua-native`
- `bare-cua-cli` to `playcua-cli`
- `bare-cua-mcp` to `playcua-mcp`
- `bare_cua_native` to `playcua_native`
- `bare_cua` to `playcua`
