# AGENTS.md — PlayCua

- **Location:** /Users/kooshapari/CodePro

## Quick Links

- **Local CLAUDE.md:** See `CLAUDE.md` in this repository for project-specific guidance
- **Phenotype org governance:** `/Users/kooshapari/CodeProjects/Phenotype/repos/CLAUDE.md`
- **Global agent guidance:** `~/.claude/AGENTS.md`
- **AgilePlus work tracking:** `cd /repos/AgilePlus && agileplus <command>`

## Key Workflows

1. **Before implementing:** Check AgilePlus for existing specs
2. **Quality gates:** Run linters, tests, and docs validation (see CLAUDE.md)
3. **Worktrees:** Use `repos/PlayCua-wtrees/<topic>/` for feature work
4. **Integration:** Commit to canonical repo (`main`) after quality gates pass

## Project-Specific Gotchas

See CLAUDE.md for language stack, build commands, and testing requirements.

---

## Active DAG
- **V3 DAG:** `FLEET_DAG_v3.db` (Phenotype org task graph)
- **Current focus:** L5 #88 — Focus-repo README + AGENTS.md standardization

---

## Architecture Decision Records

PlayCua documents architecture decisions as individual ADR files in `docs/adr/`:

| ID | Title | Status | Location |
|----|-------|--------|----------|
| ADR-006 | Modality Abstraction --- Pluggable Execution Environment | Accepted | [`docs/adr/ADR-006-modality-abstraction.md`](docs/adr/ADR-006-modality-abstraction.md) |
| ADR-007 | WINE-Bridge Research and Interop Strategy | Research Complete | [`docs/adr/ADR-007-wine-bridge-research.md`](docs/adr/ADR-007-wine-bridge-research.md) |
| ADR-008 | MCP Server + CLI as First-Class Surfaces | Accepted | [`docs/adr/ADR-008-mcp-and-cli-surfaces.md`](docs/adr/ADR-008-mcp-and-cli-surfaces.md) |

---

**Parent contract:** Extends Phenotype-org governance. See `CLAUDE.md` and parent `AGENTS.md` for complete operating procedures.
