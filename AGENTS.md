# AGENTS.md — PlayCua

## Project Location

- **Repository root:** `/Users/kooshapari/CodeProjects/Phenotype/repos/playcua`

## Quick Links

- **Local CLAUDE.md:** See `CLAUDE.md` in this repository for project-specific guidance
- **Phenotype org governance:** `/Users/kooshapari/CodeProjects/Phenotype/repos/CLAUDE.md`
- **Global agent guidance:** `~/.claude/AGENTS.md`
- **AgilePlus work tracking:** `cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus && agileplus <command>`

## Key Workflows

1. **Before implementing:** Check AgilePlus for existing specs
2. **Quality gates:** Run the local Taskfile targets in this repo (`task lint`, `task test`, `task audit`, `task docs` as needed)
3. **Worktrees:** Use `repos/playcua-wtrees/<topic>/` for feature work
4. **Integration:** Keep canonical repo aligned to `main` and commit focused fixes

## Project Structure

- `native/` — Rust native automation binary and core runtime
- `python/` — Python package and integration tests
- `bindings/` — Additional language bindings
- `contracts/` — OpenRPC contract and API schema
- `docs/` — Project documentation and worklogs
- `.github/workflows/` — CI and security workflows
- `tests/` — repository-level tests and fixtures

## Project-Specific Gotchas

See `CLAUDE.md` for language stack, build commands, and testing requirements.

---

**Parent contract:** Extends Phenotype-org governance. See `CLAUDE.md` and parent `AGENTS.md` for complete operating procedures.
