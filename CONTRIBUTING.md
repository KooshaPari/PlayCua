# Contributing to PlayCua

Thanks for your interest in contributing to **PlayCua** — part of the
[Phenotype](https://github.com/KooshaPari) ecosystem. PlayCua is a unified
computer-use agent runtime that wraps Playwright, Selenium, and bare-cua
into a single, plugin-driven SDK with native / sandbox / nvms / wsl /
container modality support.

This document explains how to set up your development environment, run
the test suite, propose changes, and get them merged safely.

1. Fork or branch from the latest main branch.
2. Make a focused change with tests or documentation updates when relevant.
3. Run the project checks that apply to your change.
4. Open a pull request with a clear summary and validation notes.

## 1. Code of Conduct

By participating, you agree to abide by the
[Phenotype Code of Conduct](CODE_OF_CONDUCT.md) (if present) and the
GitHub Community Guidelines. Be respectful, assume good faith, and
prefer written communication that can be quoted later.

## 2. Project Overview

PlayCua is a Rust-first, polyglot-bridge agent runtime. It exposes a
common `MethodPlugin` trait that downstream computer-use libraries
(Playwright, Selenium, bare-cua) implement, and routes calls to a
selected **modality** at runtime:

- `native` — direct OS process, full feature set
- `sandbox` — Apple Seatbelt / Linux user-namespace isolation
- `nvms` — NanoVMS WASM runtime
- `wsl` — Windows Subsystem for Linux
- `container` — OCI container (Docker, Podman, containerd)

The repository is a Cargo workspace (root `Cargo.toml`) plus
TypeScript bindings under `bindings/`, a Python bridge under
`python/`, and a `skills/` directory for the agent skill SDK.
The `Justfile` at the root is the canonical entry point for all
build / test / lint tasks.

## 3. Development Environment

### 3.1 Required Toolchains

| Tool             | Version   | Why                                |
|------------------|-----------|------------------------------------|
| Rust             | `stable`  | Core runtime                      |
| `cargo`          | ≥ 1.78    | Build, test, fmt, clippy           |
| `rustfmt`        | stable    | Formatting                         |
| `clippy`         | stable    | Lints (CI fails on warnings)       |
| `cargo-deny`     | ≥ 0.14    | License + advisory gating          |
| `cargo-audit`    | ≥ 0.20    | Vulnerability scan                 |
| `cargo-nextest`  | ≥ 0.9     | Faster test runner (optional)      |
| Node.js          | ≥ 20 LTS  | TS bindings                        |
| `pnpm`           | ≥ 9       | TS package manager                 |
| Python           | 3.11+     | Python bridge                      |
| `uv`             | ≥ 0.4     | Python env + dep manager           |
| `ruff`           | ≥ 0.5     | Python linter + formatter          |
| `mypy`           | ≥ 1.10    | Python type-check                  |
| `just`           | ≥ 1.36    | Task runner (preferred over Make)  |
| `lefthook`       | ≥ 1.6     | Git hooks manager                  |
| `trunk`          | ≥ 0.20    | Multi-language formatter (CI)      |

### 3.2 Clone + Bootstrap

```bash
git clone https://github.com/KooshaPari/playcua.git
cd playcua
just bootstrap
```

`just bootstrap` will:

1. Install `lefthook` git hooks (`pre-commit`, `pre-push`).
2. Install `cargo` subcommands we use (`deny`, `audit`, `nextest`,
   `outdated`, `bloat`).
3. Run `cargo fetch` and the smoke build of every workspace member.
4. (Optional) set up the `python/` venv via `uv venv` and `uv pip sync`.
5. (Optional) `pnpm install` for the TS bindings.
6. (Optional) install the modality providers (Docker, WSL, NVMS).

### 3.3 Editor Setup

- **VS Code**: open `playcua.code-workspace` (if present) or just the
  root; the recommended extensions are:
  `rust-lang.rust-analyzer`, `tamasfe.even-better-toml`,
  `charliermarsh.ruff`, `ms-python.mypy`,
  `ms-playwright.playwright`.
- **Neovim / Helix / Zed**: zero-config LSPs; the `rust-analyzer`
  config lives at `.config/rust-analyzer.toml`.
- **JetBrains RustRover**: open the root, and RustRover will pick up
  the workspace members automatically.

## 4. Building

```bash
# Everything (Rust workspace + TS bindings + Python bridge)
just build

# Just the Rust workspace
cargo build --workspace --all-targets

# Release-mode binaries
cargo build --release --workspace

# TypeScript bindings
(cd bindings && pnpm build)

# Python bridge
(cd python && uv build)
```

Useful binary outputs:

- `target/release/playcua` — main CLI.
- `target/release/playcuad` — long-running daemon.
- `target/release/pc-fleet` — bulk-fleet runner.

## 5. Testing

| Tier          | Command                                       | Owner       | Wall-clock |
|---------------|-----------------------------------------------|-------------|------------|
| Unit (Rust)   | `cargo test --workspace`                      | Core team   | < 2 min    |
| Unit (TS)     | `pnpm --filter ./bindings test`               | Bindings    | < 1 min    |
| Unit (Python) | `uv run pytest` (in `python/`)                | Bridge      | < 1 min    |
| Integration   | `just test-integration`                       | Core team   | < 10 min   |
| Snapshot      | `cargo insta test --workspace --review`       | Core team   | < 2 min    |
| Modality E2E  | `just test-modality`                          | Runtime     | < 15 min   |
| Property      | `cargo test --features proptest`              | Core team   | < 5 min    |
| Fuzz          | `cargo +nightly fuzz run parser -- -max_total_time=600` | Security | 10 min |
| E2E           | `just test-e2e`                               | Core team   | < 30 min   |

CI runs unit + snapshot + integration + property on every PR. Fuzz
and E2E run nightly and on release tags.

## 6. Coding Standards

- **Rust**: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`.
  Use `tracing` for structured logs; never `eprintln!` in library code.
- **Errors**: `thiserror` for typed error enums, `anyhow` only at the
  binary boundary. Wrap context with `.with_context()`.
- **Public APIs**: every public function has a doc-comment and a
  `#[non_exhaustive]` attribute on enums where we may add variants.
- **TypeScript**: `prettier --check`, `eslint`, `tsc --noEmit`.
- **Python**: `ruff format`, `ruff check`, `mypy --strict`. Type
  hints are mandatory on all new code.
- **Tests**: name files `<module>.test.rs` colocated with the module
  they test; integration tests live in `tests/` at the workspace root.
- **Commits**: conventional commits — see §9.

## 7. Branching

- Default branch: `main`.
- Long-lived integration branches: `release/X.Y`.
- Feature / fix / chore branches: `<type>/<scope>-<short-desc>`
  (kebab-case, ≤ 60 chars). The `<type>` matches the conventional
  commit type and the `<scope>` matches the commit scope.
  Examples: `feat/method-plugin-skill`, `fix/wsl-spawn-race`,
  `chore/l2-30-governance-2026-06-11`.

## 8. Pull Request Process

1. **Open an issue first** for non-trivial changes. Bug fixes and
   documentation improvements may go straight to PR.
2. **Fork** the repo (or push to a feature branch if you have write
   access via the Phenotype org).
3. **Keep PRs focused**: < 400 lines diff where possible. Split
   larger refactors into a stack of dependent PRs.
4. **Fill the PR template** — it links to the design doc / spec /
   issue, the test plan, and the rollout / risk notes.
5. **Pass CI**: fmt, clippy, all tier-1 tests, `cargo deny` (license +
   advisory), `cargo audit`, CodeQL, OpenSSF Scorecard check.
6. **Request a review** from the CODEOWNERS — for PlayCua the
   default reviewer is `@KooshaPari`. Add a domain reviewer (e.g.
   security, modality, bindings) for cross-cutting changes.
7. **Address review feedback** in additional commits; the maintainer
   will squash-merge once the conversation is resolved.
8. **After merge**, delete the source branch.

## 9. Commit Message Format (Conventional Commits)

PlayCua uses [Conventional Commits 1.0.0](https://www.conventionalcommits.org/).

```
<type>(<scope>): <short summary>

<body — wrap at 72 cols; explain *what* and *why*>

<footer — e.g. "BREAKING CHANGE: ...", "Closes #123", "Refs: SPEC-42">
```

### Allowed types

| Type       | Semantics                                                    |
|------------|--------------------------------------------------------------|
| `feat`     | A new user-facing feature                                    |
| `fix`      | A bug fix                                                    |
| `docs`     | Documentation only                                           |
| `style`    | Whitespace/formatting, no code change                        |
| `refactor` | Code change that neither fixes a bug nor adds a feature      |
| `perf`     | Performance improvement                                      |
| `test`     | Add or correct tests                                         |
| `build`    | Build system, CI, or dependency change                       |
| `chore`    | Tooling, repo hygiene, governance (this PR)                  |
| `revert`   | Reverts a previous commit (include `Reverts: <sha>`)         |
| `security` | Security fix (also notify `security@phenotype.internal`)     |

### Scopes (non-exhaustive)

`runtime`, `plugin`, `modality`, `agent`, `parser`, `rpc`,
`bindings`, `python`, `ci`, `docs`, `deps`, `governance`.

### Examples

```
feat(plugin): add MethodPlugin::invoke_with_retry helper

Previously a transient modality failure would surface as a
hard PluginError, which forced every plugin author to reinvent
the retry loop. The new helper centralises exponential backoff
and surface-level error classification, with a default policy
of 3 retries over 250ms.

Adds a unit test under `crates/runtime/tests/plugin_retry.rs`.
Closes #142
Refs: SPEC-12 §3
```

```
fix(modality): reject empty worktree spec with a clear error

Empty `<worktree></worktree>` blocks used to produce a
`None`-propagation panic deep inside the parser. We now surface
a `ParseError::EmptyWorktree` with the offending line number.

Fixes #487
```

## 10. Reviewer Expectations

- **First response** within 2 business days.
- Reviews cover: correctness, test coverage, security, performance,
  API stability, observability, and documentation.
- Maintainer privilege: squash-merge with the PR title as the squash
  subject and the PR body as the squash body. Override only when the
  history itself is meaningful (rare; discuss in the PR).

## 11. Release Process

PlayCua follows semver. Releases are cut from `main` by the
release-please GitHub App configured in
`.github/release-please-config.json`. The maintainer approves the
release PR, which is auto-generated and bumps versions, CHANGELOG,
and tags.

## 12. Getting Help

- **Discord**: `#playcua` on the Phenotype Discord.
- **Discussions**: GitHub Discussions → *Q&A*.
- **Office hours**: Tuesdays 15:00 UTC, calendar link in the
  pinned issue.

Welcome aboard — we are glad you are here.
