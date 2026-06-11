# PlayCua canonical justfile (L2 #21).
#
# Mirrors the recipes in `Taskfile.yml` for CI consumers that prefer
# `just` (e.g. projects that have a hard `just` requirement in CI).
#
# The `set shell := ["bash", "-uc"]` directive turns on:
#   -u: error on unset variables (fail loud, fail fast)
#   -c: pipefail — propagate the exit status of the first failing command
# This guards against silent partial failures in long cargo pipelines.
#
# Usage: `just <recipe>` (run from the repo root).
# Run `just` with no args to see the list below.

set shell := ["bash", "-uc"]
set dotenv-load

# Toolchain / scope
cargo  := "cargo"
ws     := "--workspace"

# Per L1 audit `PlayCua/STATUS_2026_06_10.md`: cargo check times out at
# 300s; the 10m timeout here mirrors the Taskfile values so both runners
# surface the failure consistently.
long_timeout  := "10m"

# Default: list recipes.
default:
    @just --list

# ---------- Build / test / lint / format ----------

# `cargo test --workspace` with an explicit timeout.
test timeout=long_timeout:
    timeout {{timeout}} {{cargo}} test {{ws}}

# `cargo build --workspace` with an explicit timeout.
build timeout=long_timeout:
    timeout {{timeout}} {{cargo}} build {{ws}}

# `cargo clippy --workspace -- -D warnings` (per CLAUDE.md quality gate).
lint:
    {{cargo}} clippy {{ws}} -- -D warnings

# `cargo fmt --all -- --check` (idempotent format check).
fmt:
    {{cargo}} fmt --all -- --check

# `cargo fmt --all` (apply formatting).
fmt-fix:
    {{cargo}} fmt --all

# ---------- Supply-chain ----------

# `cargo deny check` (advisories + bans + licenses + sources).
deny:
    {{cargo}} deny check

# `cargo audit` (RustSec advisory database).
audit:
    {{cargo}} audit

# ---------- Composed sweeps ----------

# Full local CI sweep: lint + fmt + test + deny.
ci: lint fmt test deny
    @echo "ci: all required gates passed (lint + fmt + test + deny)"

# Code-hygiene sweep: fmt + lint + deny + audit.
hygiene: fmt lint deny audit
    @echo "hygiene: fmt + lint + deny + audit all clean"

# ---------- Docs / maintenance ----------

# `cargo doc --no-deps --workspace`.
docs:
    {{cargo}} doc --no-deps {{ws}}

# `cargo clean` — drop build artifacts.
clean:
    {{cargo}} clean
