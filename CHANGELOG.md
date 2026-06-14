# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial CHANGELOG scaffold migrated to Keep-a-Changelog 1.1.0 format (L2 #36).
- Dual-license declaration via `LICENSE` (MIT) + `LICENSE-APACHE` (Apache-2.0)
  (L2 #36).

### Changed
- Migrated pre-existing changelog content into the `[Pre-format migration]`
  section below; all future entries should follow the Keep-a-Changelog
  `### Added` / `### Changed` / `### Deprecated` / `### Removed` / `### Fixed` /
  `### Security` taxonomy under `[Unreleased]`.

### Deprecated

### Removed

### Fixed

### Security

[Unreleased]: https://github.com/KooshaPari/PlayCua/compare/master...HEAD

## [Pre-format migration]

Historical entries preserved verbatim from the pre-Keep-a-Changelog
changelog scaffold. These commits predate the L2 #36 migration; their
semantic intent is captured by the bullet grouping below.

### 📚 Documentation
- Docs: add SPEC.md (`ab9d42a`)

### 🔨 Other
- Chore(deps): align tokio + serde to org baseline (phenotype-versions.toml)
  - tokio: unified to 1.39
  - serde: unified to 1.0
  - Verified: cargo check passed
  - Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com> (`b3c1331`)
- Chore(governance): adopt standard CLAUDE.md + AGENTS.md + worklog (wave-2)
  (`74007ae`)
- Test(smoke): seed minimal smoke test — proves harness works (`166fb00`)
- Chore(ci): adopt phenotype-tooling quality-gate + fr-coverage (`3cf486b`)
- Chore: add AgilePlus scaffolding (`8c18b05`)
- Ci(legacy-enforcement): add legacy tooling anti-pattern gate (WARN mode)

  Adds legacy-tooling-gate.yml monitoring per CLAUDE.md Technology Adoption
  Philosophy. Refs: phenotype/repos/tooling/legacy-enforcement/ (`6081ace`)
