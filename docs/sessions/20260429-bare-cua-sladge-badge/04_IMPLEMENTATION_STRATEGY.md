# Implementation Strategy

## Approach

Keep the badge change small and docs-only:

- README receives the sladge badge below the title.
- Session docs capture why the isolated worktree was required.
- No Rust core, Python binding, workflow, fuzz, benchmark, or native test
  changes.

## Rationale

bare-cua already had broad unrelated local work. A separate worktree allows the
sladge WBS item to be prepared and committed without disturbing that state.
