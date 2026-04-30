# Architecture Decisions

## Active ADRs

- [ADR-001: Hexagonal Architecture with JSON-RPC 2.0 IPC](./docs/adr/001-hexagonal-architecture.md)
- [ADR-002: Platform Adapter Selection Strategy](./docs/adr/002-platform-adapter-selection.md)
- [ADR-003: Plugin System Architecture](./docs/adr/003-plugin-system.md)

## ADR Status Legend

| Status | Description |
|--------|-------------|
| Proposed | Under discussion |
| Accepted | Approved and being implemented |
| Deprecated | No longer applicable |
| Superseded | Replaced by newer ADR |

## How to Create an ADR

```bash
# Create new ADR
echo "# ADR-XXX: Title

## Status
Proposed

## Context
Context here...

## Decision
Decision here...

## Consequences
Consequences here...
" > docs/adr/XXX-title.md
```