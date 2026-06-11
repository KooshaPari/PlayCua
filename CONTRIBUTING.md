# Contributing to PlayCua

Thanks for your interest in contributing to **PlayCua** — part of the
[Phenotype](https://github.com/KooshaPari) ecosystem. PlayCua is a unified
computer-use agent runtime that wraps Playwright, Selenium, and bare-cua
behind a single port/adapter seam (see `SPEC.md` and `ARCHITECTURE.md`).

This document is the canonical contributor guide. It supersedes any
shorter `CONTRIBUTING.md` you may find in older branches. If you are a
background agent, also see `AGENTS.md` and `CLAUDE.md` in this repo for
operating procedures.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Project Layout](#project-layout)
3. [Prerequisites](#prerequisites)
4. [Development Setup](#development-setup)
5. [Build](#build)
6. [Test](#test)
7. [Lint, Format, and Quality Gates](#lint-format-and-quality-gates)
8. [Coverage](#coverage)
9. [Commit Message Format (Conventional Commits)](#commit-message-format-conventional-commits)
10. [Branch and PR Process](#branch-and-pr-process)
11. [Code Review](#code-review)
12. [Reporting Issues](#reporting-issues)
13. [Security Disclosures](#security-disclosures)
14. [License](#license)

---

## Code of Conduct

By participating, you agree to abide by the [Phenotype Code of
Conduct](https://github.com/KooshaPari/phenotype-org-governance/blob/main/CODE_OF_CONDUCT.md).
Be respectful. Assume good intent. Keep technical disagreement on the
technical merits.

## Project Layout

```
PlayCua/
├── src/                 # Core library code (Rust + thin TS/Go shims)
│   ├── adapters/        # Playwright / Selenium / bare-cua adapters
│   ├── ports/           # Port traits (Renderer, Driver, Orchestrator)
│   ├── domain/          # Domain types (Session, Frame, Intent)
│   └── lib.rs
├── tests/               # Integration tests + insta snapshots
├── examples/            # Runnable example plugins (echo, screenshot, …)
├── docs/                # Long-form docs (VitePress source)
├── .github/
│   ├── workflows/       # CI, scorecard, secret-scan, dependabot
│   ├── CODEOWNERS       # Per-area ownership
│   └── FUNDING.yml      # Sponsor links
├── Cargo.toml           # Rust workspace root
├── package.json         # TS shim + VitePress docs
├── go.mod               # Bare-cua interop bridge (sub-crate)
├── SPEC.md              # Functional spec
├── ARCHITECTURE.md      # Port/adapter diagram + dependency graph
├── AGENTS.md            # Agent operating procedures
├── CLAUDE.md            # Code-agent quickstart
├── CHANGELOG.md         # Keep-a-Changelog 1.1.0
├── CODEOWNERS           # Root-level ownership alias
├── CONTRIBUTING.md      # This file
├── SECURITY.md          # Security policy
└── LICENSE              # MIT OR Apache-2.0
```

## Prerequisites

- **Rust** 1.78+ (install via [rustup](https://rustup.rs))
- **Node.js** 20+ (for VitePress docs and TS shim)
- **pnpm** or **bun** (project uses both — pick one; CI uses pnpm)
- **Go** 1.23+ (only required for the bare-cua interop bridge)
- **git** 2.40+
- A POSIX shell (`zsh` or `bash`)

Verify your toolchain:

```bash
rustc --version    # rustc 1.78.0 or newer
cargo --version
node --version     # v20.0.0 or newer
pnpm --version     # 9.x
go version         # go1.23 or newer
git --version
```

## Development Setup

```bash
# 1. Clone (substitute the fork URL if you forked first)
git clone https://github.com/KooshaPari/PlayCua.git
cd PlayCua

# 2. Install JS deps
pnpm install      # or: bun install

# 3. Pre-fetch Rust deps (faster first build)
cargo fetch

# 4. Build everything once
cargo build --workspace

# 5. Run the smoke tests
cargo test --workspace --no-run
```

The first `cargo build` is slow (~3-5 min on a clean machine); subsequent
builds are incremental.

### Recommended shell aliases

```bash
alias pc='cd /path/to/PlayCua'
alias ptest='cargo test --workspace'
alias plint='cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all -- --check'
```

## Build

PlayCua is a Cargo workspace. Standard commands apply:

```bash
# Debug build (default)
cargo build --workspace

# Release build
cargo build --workspace --release

# A single crate
cargo build -p playcua-core

# With a specific adapter feature
cargo build --no-default-features --features=playwright-adapter
cargo build --no-default-features --features=selenium-adapter
cargo build --no-default-features --features=bare-cua-adapter

# All adapters (default for CI)
cargo build --workspace --all-features
```

Build artifacts land in `target/debug/` or `target/release/`.

## Test

```bash
# Unit + integration tests
cargo test --workspace

# A single test by name
cargo test --workspace session::frame::resize

# Doc tests
cargo test --workspace --doc

# With race detection (where supported)
cargo test --workspace -- --test-threads=1

# Snapshot updates (insta)
cargo insta review          # interactive
cargo insta accept          # non-interactive
cargo insta reject
```

Test output is ANSI-colored; prefix with `NO_COLOR=1` for plain logs.

## Lint, Format, and Quality Gates

All gates MUST pass before pushing. The CI runs the same set on every PR.

```bash
# Format check (auto-fixes on save in most editors)
cargo fmt --all -- --check

# Lints (deny warnings, fail on clippy::pedantic)
cargo clippy --workspace --all-targets -- -D warnings

# cargo-deny (license + advisory + ban + source)
cargo deny check

# cargo-audit (RustSec advisory database)
cargo audit

# Pre-commit (gitleaks + trufflehog + fmt + clippy)
pre-commit run --all-files
```

CI also runs `cargo llvm-cov` and posts the line-coverage delta as a PR
comment. **Coverage regressions on changed lines are a soft fail** — the
PR may be merged if justified, but the author is asked to add tests in a
follow-up.

## Coverage

```bash
# HTML report
cargo llvm-cov --workspace --html --output-dir coverage/

# LCOV (for CI)
cargo llvm-cov --workspace --lcov --output-path lcov.info

# Summary
cargo llvm-cov report --summary
```

The line-coverage target on the 3 main crates (`playcua-core`,
`playcua-cli`, `playcua-mcp`) is **≥ 80%**. Drops below 70% require a
justification in the PR body.

## Commit Message Format (Conventional Commits)

PlayCua uses [Conventional Commits 1.0.0](https://www.conventionalcommits.org/).

### Format

```
<type>(<scope>): <short summary>

<body — wrap at 72 columns>

<footer>
```

### Allowed types

| Type       | Purpose                                                  |
|------------|----------------------------------------------------------|
| `feat`     | A new user-visible feature                               |
| `fix`      | A bug fix                                                |
| `docs`     | Documentation only                                       |
| `style`    | Formatting (no logic change)                             |
| `refactor` | Code restructuring (no behavior change)                  |
| `perf`     | Performance improvement                                  |
| `test`     | Adding or fixing tests                                   |
| `build`    | Build system / dependency change                         |
| `ci`       | CI configuration                                         |
| `chore`    | Maintenance, tooling, governance (no source change)      |
| `revert`   | Revert a previous commit                                 |

### Scopes (recommended)

`core`, `cli`, `mcp`, `adapter-playwright`, `adapter-selenium`,
`adapter-bare-cua`, `domain`, `ports`, `docs`, `ci`, `governance`.

### Examples

```
feat(adapter-playwright): add retry-on-stale-element handler

fix(core): clamp session timeout to [1s, 24h]

docs(arch): add sequence diagram for the Driver port

chore(governance): add CODEOWNERS, CONTRIBUTING, SECURITY, FUNDING (L2 #30)
```

### Breaking changes

Mark with `!` after the type/scope and a `BREAKING CHANGE:` footer:

```
feat(api)!: rename SessionHandle.id to SessionHandle.handle

BREAKING CHANGE: callers must use `.handle` instead of `.id`.
Migration: rg '\.id\b' --type rust | xargs sed -i '' 's/\.id\b/.handle/g'
```

## Branch and PR Process

### Branch naming

- `feat/<short-kebab>` — new user-visible feature
- `fix/<short-kebab>` — bug fix
- `chore/<short-kebab>` — maintenance, deps, governance
- `docs/<short-kebab>` — documentation
- `refactor/<short-kebab>` — code restructure
- `hotfix/<short-kebab>` — urgent production fix (expedited review)

### Workflow

1. **Branch** off `main`:
   ```bash
   git checkout main && git pull
   git checkout -b feat/your-feature
   ```
2. **Develop** in small, focused commits.
3. **Run the full quality gate** locally:
   ```bash
   cargo fmt --all -- --check && \
     cargo clippy --workspace --all-targets -- -D warnings && \
     cargo test --workspace && \
     cargo deny check
   ```
4. **Push** and **open a PR** against `main`:
   ```bash
   git push -u origin feat/your-feature
   gh pr create --base main --title "feat(scope): short summary" \
     --body-file .github/PULL_REQUEST_TEMPLATE.md
   ```
5. **Address review** in additional commits (do not force-push during
   review — rebase only after approval).
6. **Squash-merge** via the GitHub UI; the squash commit message MUST
   follow the conventional-commits format.

### PR requirements (CI will enforce)

- [ ] Title matches `<type>(<scope>): <summary>`
- [ ] Body references the issue / spec it closes (`Closes #123`)
- [ ] At least 1 approving review from a CODEOWNER
- [ ] All CI checks green (fmt, clippy, test, deny, audit, cov)
- [ ] No new `TODO` without a tracking issue

## Code Review

Reviewers should:

- **Be specific** — quote the line, suggest the fix, link the doc.
- **Distinguish** blocking from non-blocking (prefix with `[blocking]`
  or `[nit]`).
- **Approve explicitly** — use the GitHub "Approve" button, not a
  "Looks good" comment.

Authors should:

- **Respond to every comment** — either push a fix or explain why not.
- **Keep the diff small** — split a 1500-line PR into 3 stacked PRs.
- **Self-review first** — read your own diff on the GitHub PR view
  before requesting review.

Review SLA: 1 business day for the first round. If a reviewer is
unreachable, ping `@KooshaPari` to reassign.

## Reporting Issues

Use the GitHub issue templates under `.github/ISSUE_TEMPLATE/`. Always
include:

- PlayCua version (`playcua --version`)
- Operating system and architecture (`uname -a`)
- Rust toolchain (`rustc --version`)
- Reproduction steps (the smallest possible snippet)
- Expected vs. actual behavior
- Relevant logs (`RUST_LOG=debug playcua …`)

## Security Disclosures

For sensitive vulnerabilities, **do not open a public issue**. Follow
the process in [`SECURITY.md`](./SECURITY.md). You can expect an
acknowledgment within 48 hours and a triage decision within 7 days.

## License

By contributing, you agree that your contributions will be licensed
under the **MIT OR Apache-2.0** license (dual-licensed, at the option
of downstream consumers). See [`LICENSE`](./LICENSE) for the full text.

---

Questions? Open a discussion at
https://github.com/KooshaPari/PlayCua/discussions or reach out to
@KooshaPari on the Phenotype Discord.
