# bare-cua Specification

## Architecture
```
┌─────────────────────────────────────────────────────┐
│              Bare CUA (Rust + Python)               │
├─────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────┐   │
│  │         Native Binaries (Rust)            │   │
│  │   ┌─────────┐   ┌─────────┐   ┌────────┐  │   │
│  │   │ macOS   │   │ Linux  │   │Win64  │  │   │
│  │   └─────────┘   └─────────┘   └────────┘  │   │
│  └───────────────────────────────────────────┘   │
│  ┌───────────────────────────────────────────┐   │
│  │         Python Bindings (PyO3)            │   │
│  └───────────────────────────────────────────┘   │
└──────────────────────────────────────────────┘
```

## Components

| Component | Responsibility | Public API |
|-----------|----------------|-----------|
| native | Cross-platform CUA binary | CLI interface |
| bindings | Python wrapper | `BARE.execute()` |
| contracts | Protocol definitions | TLA+ specs |

## Data Models

```rust
struct Action {
    name: String,
    args: HashMap<String, Value>,
    timeout: Duration,
}

struct ExecutionResult {
    output: String,
    exit_code: i32,
    duration: Duration,
}
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Action exec | <5s |
| Cold start | <2s |
| Memory | <50MB |
| Platforms | macOS, Linux, Win |