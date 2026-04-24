# Changelog

All notable changes to this project will be documented in this file.

## 📚 Documentation
- Docs(wave-4): scaffold FUNCTIONAL_REQUIREMENTS.md with 6 stubs (`b86c3a6`)
- Docs(fr): scaffold FUNCTIONAL_REQUIREMENTS.md with 2 FR stubs (`ae7dd72`)
- Docs: add SPEC.md (`ab9d42a`)
- Docs: add PLAN.md (`c03290f`)
## ✨ Features
- Feat: initial bare-cua scaffold

- Hexagonal architecture: domain → ports → adapters → ipc → app
- Rust native binary (bare-cua-native) with stdio JSON-RPC
- Platform adapters: WGC (Windows), xcap (fallback), enigo (input)
- Win32 EnumWindows, SendInput/PostMessage, IGraphicsCaptureItemInterop
- Python thin client (~120 lines) + ComputerAgent loop
- C# binding (NativeComputer, IAsyncDisposable)
- Sandbox layer: Sandboxfile, SandboxConfig, WSB/Hyper-V adapters
- OpenRPC 1.2.6 contract (contracts/openrpc.json, 14 methods)
- Plugin system (MethodPlugin trait + PluginRegistry)
- Unit tests: analysis (7), Python mock subprocess (4), plugin registry (3)
- SOLID, KISS/DRY, contract-first, polyglot-polyrepo design

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com> (`332dcde`)
## 🔨 Other
- Chore(deps): align tokio + serde to org baseline (phenotype-versions.toml)

- tokio: unified to 1.39
- serde: unified to 1.0
- Verified: cargo check passed

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com> (`f259534`)
- Chore(ci): adopt phenotype-tooling workflows (wave-3) (`9dc69ab`)
- Test(smoke): seed minimal smoke test — proves harness works (`7f1573b`)
- Chore(governance): adopt standard CLAUDE.md + AGENTS.md + worklog (`2191f80`)
- Ci(legacy-enforcement): add legacy tooling anti-pattern gate (WARN mode)

Adds legacy-tooling-gate.yml monitoring per CLAUDE.md Technology Adoption Philosophy.

Refs: phenotype/repos/tooling/legacy-enforcement/ (`a6ee950`)
- Ci: migrate to reusable workflows from template-commons

- Use reusable-rust-ci.yml, reusable-python-ci.yml, reusable-typescript-ci.yml
- Add security scanning with reusable-security-scan.yml
- Add governance validation with validate-governance.yml (`04c5048`)
- Chore: add AgilePlus scaffolding (`f92138b`)
- Chore(infra): add standardized infrastructure files (`9d9b2c8`)