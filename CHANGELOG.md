# Changelog

All notable changes to this project will be documented in this file.

## 📚 Documentation
- Docs: add SPEC.md (`ab9d42a`)
## 🔨 Other
- Chore(deps): align tokio + serde to org baseline (phenotype-versions.toml)

- tokio: unified to 1.39
- serde: unified to 1.0
- Verified: cargo check passed

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com> (`b3c1331`)
- Chore(governance): adopt standard CLAUDE.md + AGENTS.md + worklog (wave-2) (`74007ae`)
- Test(smoke): seed minimal smoke test — proves harness works (`166fb00`)
- Chore(ci): adopt phenotype-tooling quality-gate + fr-coverage (`3cf486b`)
- Chore: add AgilePlus scaffolding (`8c18b05`)
- Ci(legacy-enforcement): add legacy tooling anti-pattern gate (WARN mode)

Adds legacy-tooling-gate.yml monitoring per CLAUDE.md Technology Adoption Philosophy.

Refs: phenotype/repos/tooling/legacy-enforcement/ (`6081ace`)