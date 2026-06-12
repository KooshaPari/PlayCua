# PlayCua — AGENTS.md

> **AI-agent constitution for `PlayCua`.** Generated from the V3 §120
> SD4 SOTA pattern (V18 build/test/style/do-not-touch constitution) on
> 2026-06-12. Read this fully before making changes.

---

## 1. Quick start (build, test, lint)

```bash
# Build the native Rust binary (playcua-native)
cargo build --workspace

# Run the full test suite (unit + integration + doctest)
cargo test --workspace

# Lint (zero warnings enforced; matches L1 quality gate)
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check

# Supply-chain (configured via deny.toml)
cargo deny check
cargo audit

# Convenience via just:
just ci          # lint + fmt + test + deny
just hygiene     # fmt + lint + deny + audit
```

Python bindings (`python/`) use `uv`:

```bash
cd python && uv sync && uv run pytest
```

---

## 2. Project layout (top-level dirs + purpose)

| Path | Purpose |
|------|---------|
| `native/` | Native Rust crate `playcua-native` — JSON-RPC 2.0 stdio server, hexagonal ports/adapters |
| `contracts/` | OpenRPC 1.2.6 contract (`openrpc.json`) — source of truth for the IPC API |
| `bindings/` | Generated language bindings (Python, C#, …) |
| `python/` | Python SDK + `playcua` CLI |
| `sandbox/` | Sandbox harness / fixtures for integration tests |
| `docs/` | Public docs (architecture, OpenRPC, port catalog) |
| `scripts/` | Repository automation |
| `.devcontainer/` | Reproducible dev container |
| `.github/workflows/` | 13 CI workflows (quality-gate, codeql, cargo-deny, secret-scan, …) |

Hexagonal architecture: domain types in `native/playcua-core/` are pure Rust
structs with zero external deps; ports are async traits; adapters
(`xcap`, `enigo`, `windows-capture`, `core-graphics`) are swappable
implementations selected at compile time.

---

## 3. Key files (entry points, config files)

| File | Role |
|------|------|
| `Cargo.toml` | Workspace manifest (members = `["native"]`) |
| `Cargo.lock` | Pinned dependency graph |
| `justfile` | Canonical task runner (L2 #21; `set shell := ["bash", "-uc"]` for pipefail) |
| `rust-toolchain.toml` | Stable toolchain pin |
| `rustfmt.toml`, `clippy.toml` | Format + lint config |
| `deny.toml` | cargo-deny advisories/bans/licenses/sources |
| `.pre-commit-config.yaml` | pre-commit hooks (rustfmt, clippy, cargo-machete, trufflehog, gitleaks) |
| `.gitleaks.toml`, `.trufflehog.yml` | Secret-scanning config |
| `CODEOWNERS` | Per-path review routing |
| `cliff.toml` | git-cliff release-notes template |
| `renovate.json5` | Renovate bot dependency-update config |
| `CHANGELOG.md` | Release history |
| `CLAUDE.md` | Claude-specific operating notes |
| `contracts/openrpc.json` | IPC API contract (generate bindings from this) |
| `native/src/main.rs` | Binary entrypoint — IPC loop (read → dispatch → write) |
| `native/src/ipc/dispatcher.rs` | JSON-RPC method → port call |
| `DEPRECATED_BARE_CUA.md` | Records the 2026-06-08 `bare-cua` → PlayCua merge |

---

## 4. Conventions

- **Commit message format** — Conventional Commits, scope = crate or
  concern: `feat(playcua-native): …`, `fix(playcua-core): …`, `feat(openrpc): …`,
  `docs(playcua): …`. The `native` workspace member is the dominant scope.
- **Branch naming** — `<prefix>/<TID>-<topic>-<date>` where prefix ∈
  `{feat, fix, chore, ci, docs, refactor, test, perf, build}` and
  TID is a V3 DAG task ID (e.g. `L1-002`, `CC2-002`, `SD4`). Examples:
  `chore/L1-007-sota-screenshot-png-2026-06-11`,
  `feat/L2-013-bon-builder-2026-06-11`,
  `chore/SD4-2026-06-12` (this worktree).
- **Worklog schema** — V2 10-column JSON schema. Canonical reference:
  [`pheno-worklog-schema`](https://github.com/KooshaPari/pheno-worklog-schema)
  (or local `pheno-worklog-schema/` if vendored). Each task produces
  one worklog JSON at the repo root: `worklog-<TID>-<topic>.json`.
- **PR policy** — `master` is protected (1 reviewer required, no force-push).
  All changes flow through PRs.
- **IPC contract** — every new JSON-RPC method MUST be added to
  `contracts/openrpc.json` first; the dispatcher is generated from it.
- **Encoding** — UTF-8, no BOM. Never commit agent dirs.

---

## 5. Common tasks

### Add a Rust dependency to `playcua-native`

```bash
# From the workspace root
cargo add -p playcua-native <crate-name> --features <feature>

cargo build --workspace
cargo deny check       # license + bans + advisories + sources
```

### Add a Rust test

- **Unit test** — `#[cfg(test)] mod tests` block at the bottom of the same
  file (idiomatic, no extra setup).
- **Integration test** — new file under `native/tests/<topic>.rs`. Use
  `sandbox/` for shared fixtures and `wiremock` (already a dev-dep) for
  IPC round-trip tests.
- **Doctest** — when the example is non-trivial, add a runnable example
  to a `///` doc comment; the CI matrix runs doctests.
- **Snapshot test** — use `insta` (already a dev-dep) for any large
  binary output (e.g. PNG byte streams).

### Run benchmarks

```bash
cargo bench --workspace
# HTML report: target/criterion/report/index.html
```

---

## 6. Tooling

- **Task runner: `justfile` (casey/just).** Chosen for the L2 #21 SOTA
  because: (1) casey/just is the org-wide standard (mirrors AgilePlus,
  nanovms, PhenoCompose, BytePort), (2) the L2 spec mandates a
  `set shell := ["bash", "-uc"]` directive so that pipefail + unset-variable
  detection surface long cargo-pipeline failures loudly, (3) recipes like
  `just ci` = `lint + fmt + test + deny` compose cleanly.
- **Linter: `cargo clippy --workspace -- -D warnings`** (CI-enforced).
- **Formatter: `cargo fmt --all -- --check`** (CI-enforced).
- **Pre-commit: pre-commit framework** (`.pre-commit-config.yaml`) running
  rustfmt, clippy, cargo-machete, trufflehog, gitleaks. Install with
  `brew install pre-commit && pre-commit install`.
- **Supply-chain: `cargo-deny` + `cargo-audit`** (deny.toml +
  `cargo-deny.yml` + `cargo-audit.yml` workflows + weekly
  `rustsec/audit-check@v2`).
- **Coverage: `cargo llvm-cov` + Codecov** (in `quality-gate.yml`).
- **Semver: `cargo-semver-checks`** (in CI).
- **Releases: `git-cliff`** (cliff.toml) → tags + GitHub release notes.
- **VCS: git worktrees** — work in `PlayCua-wtrees/<topic>/`, never
  directly in the canonical `PlayCua/` checkout on `master`.

---

## 7. Do not touch (without an explicit task)

- `Cargo.toml [workspace.members]` — adding/removing members is an L2 SOTA task.
- `rust-toolchain.toml` — toolchain pin is contractual.
- `deny.toml`, `clippy.toml`, `rustfmt.toml` — version pins are intentional.
- `contracts/openrpc.json` — the contract is the source of truth; any
  method removal is a breaking IPC change.
- `native/src/ipc/dispatcher.rs` — generated from the contract; do not hand-edit.
- The `playcua` Python/C# bindings — re-generated from the contract
  after the canonical Rust dispatcher is updated.
- The `.pre-commit-config.yaml` `trufflehog` hook id — replaced by the
  `phenotype-secret-scan` workflow in a future L2 pass.
- `CODEOWNERS` — review routing is governance-mandated.

---

## 8. Reference

- **V3 §120 (SD4 SOTA pattern)** — this file's section layout.
- **V18 §110 pheno-otel AI-DD crutches** — the 5-convention-file pattern
  (AGENTS.md, llms.txt, WORKLOG.md, CHANGELOG.md, LICENSE-MIT) inherited
  per-crate.
- **V11 §70.3 (AX/L16 acceptance)** — `cargo clippy --workspace -- -D warnings`
  is the canonical zero-warnings gate.
- **FLEET_100TASK_DAG_V3.md** — task IDs (`L1-002`, `CC2-002`, `SD4`).
- **CLAUDE.md** — Claude-specific operating notes.
- **DEPRECATED_BARE_CUA.md** — 2026-06-08 merge record (the `bare-cua`
  repo is frozen at the 2026-06-08 snapshot).
- **phenotype-org-governance/SUPERSEDED.md** — governance authority
  (when present, supersedes local conventions).

---

## 9. License

MIT OR Apache-2.0 (dual). See `LICENSE`, `LICENSE-MIT`, and `LICENSE-APACHE`.
Copyright 2026 Koosha Pari.
