# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

The latest `main` branch is also considered supported. Older tagged
releases receive security fixes on a best-effort basis; please upgrade
to the latest patch release of your minor version.

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Use one of the following channels (in order of preference):

1. **GitHub Security Advisories (private):**
   <https://github.com/KooshaPari/PlayCua/security/advisories/new>
2. **Email:** `security@phenotype.dev` (PGP key on request). Fallback:
   `kooshapari@gmail.com`.
3. **DM:** @KooshaPari on the Phenotype Discord.

### What to include

- A clear, concise description of the vulnerability
- A proof-of-concept (snippets, screenshots, or a minimal reproducer)
- The affected versions / commits
- The potential impact (data exposure, RCE, DoS, etc.)
- Any suggested mitigations or fixes (optional but appreciated)
- Whether you intend to disclose publicly (and when)

### Response timeline

| Stage                       | SLA                |
|-----------------------------|--------------------|
| Acknowledgment              | 48 hours           |
| Initial triage + severity   | 7 days             |
| Patch for CRITICAL/HIGH     | 7 / 30 days        |
| Patch for MEDIUM/LOW        | Next release cycle |
| Public advisory (after fix) | Coordinated        |

CRITICAL: actively exploitable, full RCE, auth bypass, or data loss on
default configuration. HIGH: exploitable with low complexity or
significant impact. MEDIUM/LOW: limited impact, requires uncommon
configuration, or purely theoretical.

## Disclosure Policy

We follow **coordinated disclosure**. Once a fix is available (or after
90 days, whichever comes first) we will:

1. Publish a GitHub Security Advisory with the CVE, the affected
   versions, the patched versions, and the credit.
2. Cut a patch release (`x.y.z+1`).
3. Update the CHANGELOG with a `[SECURITY]` entry.
4. Notify known downstream consumers via the Phenotype Discord
   `#security` channel.

We will credit the reporter unless they ask to remain anonymous.

## Security Tooling

- `cargo audit` — runs on every PR (RustSec advisory database)
- `cargo deny` — license + advisory + source allow-list
  (`.github/workflows/deny.yml`)
- `gitleaks` + `trufflehog` — pre-commit + scheduled secret scan
  (`.github/workflows/secret-scan.yml`)
- `codeql` — weekly static analysis (`.github/workflows/codeql.yml`)
- `scorecard` — weekly OSSF scorecard
  (`.github/workflows/scorecard.yml`)
- Dependabot — daily dependency updates (`.github/dependabot.yml`)

## Out-of-Scope

The following are **not** considered vulnerabilities in PlayCua:

- Issues in upstream dependencies that do not affect PlayCua directly
  (report to the upstream project; we will help coordinate if needed)
- Theoretical issues without a working PoC
- Issues requiring the user to install untrusted code or config
- Rate-limiting / resource-exhaustion on a single-user developer machine
  (production hardening is a separate roadmap item)

## Bug Bounty

PlayCua is a volunteer-maintained open-source project. There is **no
formal bug-bounty program** at this time, but reporters are credited in
the advisory and the CHANGELOG. Significant findings may receive a
small bounty or a Phenotype org sponsorship — at the maintainer's
discretion.

---

Thank you for helping keep PlayCua — and the broader Phenotype
ecosystem — secure.
