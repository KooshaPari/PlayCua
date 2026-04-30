# Specifications

## Scope

- Add the sladge badge to `README.md`.
- Do not change Rust code, Python bindings, Cargo files, specs, contracts, or
  generated artifacts.
- Preserve unrelated canonical checkout changes.

## Acceptance Criteria

- README includes `[![AI Slop Inside](https://sladge.net/badge.svg)](https://sladge.net)`.
- Badge appears directly under the top-level heading.
- Session docs explain why the repo is in scope.

## Assumptions, Risks, Uncertainties

- Assumption: Computer-use agent examples with Claude model selection make the
  repo materially AI-agent-related.
- Risk: Canonical merge may need to account for unrelated Cargo/spec/ADR work.
- Mitigation: Record the prepared commit and worktree in projects-landing.
