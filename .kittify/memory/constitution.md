# Project Constitution

> Auto-provisioned by phenotype-org-governance lane rollout
> Date: 2026-06-24
> Lane: computer-use (koosh)
> Pattern: mirrors agileplus `.kittify/memory/constitution.md`

## Purpose

This constitution captures the technical standards, code quality expectations,
tribal knowledge, and governance rules for this repository. All features and
pull requests should align with these principles.

## Technical Standards

### Language and Toolchain

- **Primary language**: see `rust-toolchain.toml` (or `go.mod` / `package.json`)
  - Pin to a stable channel; roll on LTS cadence
- **Formatter**: see `rustfmt.toml` (or `gofmt` / `prettier`)
  - Strict, enforced via `lefthook` pre-commit
- **Linter**: see `clippy.toml` (or `.golangci.yml` / `.eslintrc`)
  - Treat warnings as fatal in CI
- **License audit**: see `deny.toml` (Rust) — `cargo deny check` blocks release

### Governance Gates (pre-commit / pre-push / commit-msg)

See `lefthook.yml` in this repo. Standard gates:

- Pre-commit: format, lint, license-headers, secret-scan
- Pre-push: full test suite with race detection
- Commit-msg: Conventional Commits format

### Branch and PR Hygiene

- Branch names: `hygiene/<change>-<yyyymmdd>` or `feat/<slug>-<yyyymmdd>`
- One logical change per PR
- Reference: `phenotype-org-audits DAG-012` (71-pillar L0..L80)
- Reference: `phenotype-org-governance ADR-039` (fork-tracking-archive-not-delete)

## Code Quality

- Treat security warnings as fatal
- Run all required tests before claiming work complete
- State what was done, what was not, and why

## Tribal Knowledge

- This repo is part of the **computer-use domain** under
  `phenotype-org-governance`. Lane owner: koosh (forge).
- Sibling repos: Eidolon, kmobile, KDesktopVirt, KodeVibe, mobile-mpc, mobilecli,
  playcua, phenotype-journeys, Dino (archived).
- Cross-repo coordination happens via `phenotype-registry/coordination/`.

## Versioning

- SemVer for public crates
- Calendar version for governance documents (`YYYY.MM.DD`)

## Quick Reference

- Path: always specify exact locations in agent prompts
- Encoding: UTF-8 only
- Context: read what you need, don't re-read unnecessarily
- Quality: secure, tested, documented
- Git: clean commits, descriptive messages
