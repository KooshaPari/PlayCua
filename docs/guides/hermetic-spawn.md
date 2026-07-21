# Hermetic spawn test

The modality drivers (`SandboxDriver`, `NvmsDriver`, `ContainerDriver`,
`WslDriver`) share a spawn/kill pattern: `tokio::process::Command` with
piped stdio, optional `setsid` on Unix, SIGTERM then SIGKILL after 5s.

## Local

```bash
# From repo root
cargo run --locked --manifest-path native/Cargo.toml --bin hermetic_spawn_test
cargo run --locked --manifest-path native/Cargo.toml --bin hermetic_spawn_test -- --kill

# Driver + port dispatch unit tests (no host sandbox binary)
PLAYCUA_SANDBOX_BACKEND=direct cargo test --locked --manifest-path native/Cargo.toml
```

Fixtures live in `native/tests/fixtures/` (`fake-nvms.sh`, `fake-sandbox.sh`).

## CI

Workflow: `.github/workflows/hermetic-spawn.yml` (Ubuntu). If GitHub Actions
billing blocks the job, the local commands above are the source of truth —
`cargo test --locked` also covers Direct-backend spawn via
`native/tests/hermetic_sandbox_driver.rs` and `WireSandboxAdapter` tests.
