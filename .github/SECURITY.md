# Security Policy — PlayCua
#
# The canonical security policy lives at the repository root in
# `/SECURITY.md`. This file exists so that GitHub's Security tab picks up
# the policy in the conventional `.github/` location as well.
#
# Both files are kept in sync; edits should be made to the root copy and
# mirrored here. `CODEOWNERS` pins both files to @KooshaPari.

> **See the canonical security policy at [`/SECURITY.md`](../SECURITY.md).**
>
> Supported versions, reporting channels (GitHub private vulnerability
> reporting, `security@phenotype.internal`, Signal), SLOs, coordinated
> disclosure window, severity rating, and bug-bounty / recognition
> information are all defined in that document.

## Reporting a vulnerability

Use **GitHub private vulnerability reporting**:

> *Repository → Security → Advisories → "New draft security advisory"*

This is the preferred channel and gives you a private thread with the
maintainers, automatic CVE assignment, and a coordinated disclosure
workflow. See `/SECURITY.md` for the full triage and response SLOs.

## Tooling this PR adds

- `.github/workflows/audit.yml` — `cargo audit`, `npm audit`, `pip-audit`
  across the Rust / Node / Python ecosystems actually present in this
  workspace.
- `.github/workflows/deny.yml` — `cargo-deny` (advisories + licenses +
  bans + sources) plus a `govulncheck` job for `go.mod` (no-op today;
  activates the moment a Go module is added).
- `deny.toml` — cargo-deny configuration (license allow-list, advisory
  ignores, banned crates).
- `.github/dependabot.yml` — pre-existing; weekly cadence for cargo,
  npm, gomod, github-actions, docker.

This file is the `.github/` mirror; the canonical policy is at the
repository root.