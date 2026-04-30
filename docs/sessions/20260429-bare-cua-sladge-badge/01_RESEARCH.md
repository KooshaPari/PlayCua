# Research

## Repo Fit

bare-cua is in scope for the sladge rollout because its local agent contract
defines it as a Computer Use Agent framework and documents an observe, think,
act, verify CUA action flow with LLM analysis.

## Local State

Canonical `bare-cua` had unrelated local Cargo, spec, workflow, ADR, PRD,
benchmark, fuzz, native test, and worklog changes. The badge change was prepared
in an isolated worktree to avoid mixing those changes.

## Decision

Treat this as a documentation/governance badge update only. Do not modify Rust
core code, Python bindings, workflows, fuzz targets, or native tests.
