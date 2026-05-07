# bare-cua

Rust fuzzing and testing workspace for CUA (Cognitive User Assistant) patterns.

## Language Stack

- Rust (stable)

## Key Commands

```bash
cargo test --all-features --workspace
cargo clippy --all-features -- -D warnings
```

## Key Files

- `README.md`
- `CHANGELOG.md`
- `.github/workflows/ci.yml` — CI pipeline
- `.github/workflows/trufflehog.yml` — secrets scanning

## Notes

- No source files yet; this is a scaffolded repo.
- Requires `cargo-fuzz` for fuzzing targets.
