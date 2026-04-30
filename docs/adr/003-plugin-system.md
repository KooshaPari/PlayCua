# ADR-003: Plugin System for Extensible Method Dispatch

## Status

Accepted

## Context

bare-cua provides core computer-use capabilities (screenshot, input, windows, process, analysis). However, users will inevitably need custom functionality that doesn't belong in core:

1. **Domain-specific actions**: Healthcare, finance, CAD automation
2. **Integration methods**: Database queries, API calls, cloud service interaction
3. **Custom analysis**: OCR, ML inference, specialized image processing
4. **Proprietary protocols**: Internal tools, legacy systems

### Design Goals

| Goal | Priority | Description |
|------|----------|-------------|
| **Isolation** | High | Plugins must not crash the host |
| **Performance** | Medium | Minimal overhead for plugin calls |
| **Ergonomics** | High | Simple API for plugin authors |
| **Safety** | Critical | Plugins cannot escalate privileges |
| **Dynamic loading** | Low | Compile-time linking acceptable |

### Alternatives Considered

| Approach | Isolation | Performance | Ergonomics | Safety | Dynamic |
|----------|-----------|-------------|------------|--------|---------|
| Native dynamic libraries (dlopen) | Low | High | Medium | Low | Yes |
| WebAssembly (WASM) | High | Medium | Medium | High | Yes |
| WebAssembly (WASI) | High | Medium | Hard | High | Yes |
| gRPC sidecars | High | Medium | Easy | High | Yes |
| JSON-RPC child processes | High | Low | Easy | High | Yes |
| Rust trait objects (compile-time) | Medium | High | Easy | Medium | No |
| Lua/Python scripting | Medium | Low | Easy | Low | Yes |

## Decision

We implement a **compile-time plugin system** using Rust trait objects with JSON-RPC child process support planned for future phases.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Plugin System Architecture                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Phase 1: Compile-Time Plugins (Current)                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Dispatcher                                     │   │
│  │  ┌─────────────────┐                                                │   │
│  │  │  Built-in methods│  ping, screenshot, input.*, windows.*, etc.   │   │
│  │  └─────────────────┘                                                │   │
│  │            │                                                        │   │
│  │            ▼                                                        │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │                    PluginRegistry                             │   │   │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │   │   │
│  │  │  │ Plugin A    │ │ Plugin B    │ │ Plugin C    │            │   │   │
│  │  │  │ method_name │ │ method_name │ │ method_name │            │   │   │
│  │  │  │   "custom"  │ │  "db.query" │ │  "ocr.read" │            │   │   │
│  │  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘            │   │   │
│  │  └─────────┼───────────────┼───────────────┼───────────────────┘   │   │
│  │            │               │               │                        │   │
│  └────────────┼───────────────┼───────────────┼────────────────────────┘   │
│               │               │               │                             │
│  ┌────────────┴───────────────┴───────────────┴────────────────────────┐   │
│  │                    Plugin Trait Implementations                       │   │
│  │  struct MyPlugin; impl MethodPlugin for MyPlugin { ... }            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Phase 2: External Process Plugins (Future)                               │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  ┌─────────────┐    stdin/stdout    ┌─────────────────────────────┐  │   │
│  │  │  bare-cua   │◄───JSON-RPC─────►│  Plugin Process (any lang)  │  │   │
│  │  │  (host)     │    over pipes    │  - Python ML model           │  │   │
│  │  │             │                  │  - Node.js automation        │  │   │
│  │  │             │                  │  - Go cloud SDK              │  │   │
│  │  └─────────────┘                  │  - WASM runtime              │  │   │
│  │                                    └─────────────────────────────┘  │   │
│  │                                    Sandboxed via seccomp/Landlock   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Phase 3: WASM Plugins (Future)                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  ┌─────────────┐    WASM runtime    ┌─────────────────────────────┐  │   │
│  │  │  bare-cua   │◄───WASI calls────►│  WASM Module (Rust/C/Go)    │  │   │
│  │  │  (host)     │    capability     │  - Sandboxed                 │  │   │
│  │  │             │    restricted     │  - Near-native speed         │  │   │
│  │  └─────────────┘                   │  - Cross-platform binary     │  │   │
│  │                                    └─────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Phase 1: Compile-Time Plugin API

```rust
/// A plugin that handles a single JSON-RPC method name.
#[async_trait]
pub trait MethodPlugin: Send + Sync {
    /// The exact method name this plugin handles (e.g. "custom.foo").
    fn method_name(&self) -> &'static str;

    /// Handle an incoming request. `params` is the raw JSON params value
    /// (may be Null if no params were provided).
    async fn handle(&self, params: Value) -> anyhow::Result<Value>;
}

/// Registry of all registered plugins, keyed by method name.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn MethodPlugin>>,
}

impl PluginRegistry {
    /// Register a plugin. If a plugin with the same method name is already
    /// registered, the new one replaces it.
    pub fn register(&mut self, plugin: Box<dyn MethodPlugin>);

    /// Find the plugin registered for `method`, if any.
    pub fn find(&self, method: &str) -> Option<&dyn MethodPlugin>;
}
```

### Usage Example

```rust
// Define a custom plugin
struct DbQueryPlugin {
    pool: sqlx::PgPool,
}

#[async_trait]
impl MethodPlugin for DbQueryPlugin {
    fn method_name(&self) -> &'static str {
        "db.query"
    }

    async fn handle(&self, params: Value) -> anyhow::Result<Value> {
        let query: String = serde_json::from_value(params["sql"].clone())?;
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        Ok(json!({ "rows": rows.len() }))
    }
}

// Register in main
let mut registry = PluginRegistry::new();
registry.register(Box::new(DbQueryPlugin { pool }));

// Dispatcher consults registry for unknown methods
if let Some(plugin) = registry.find(method) {
    plugin.handle(params).await
}
```

### Method Resolution Order

```
┌────────────────────────────────────────┐
│         Incoming Request               │
│  {"method":"custom.action",...}       │
└──────────────┬─────────────────────────┘
               │
               ▼
┌────────────────────────────────────────┐
│     1. Check Built-in Methods          │
│     screenshot, input.*, etc.          │
│     └─▶ Match? ──Yes──▶ Execute        │
│            │                           │
│            No                          │
│            ▼                           │
│     2. Check PluginRegistry            │
│     registry.find("custom.action")     │
│     └─▶ Match? ──Yes──▶ Execute Plugin  │
│            │                           │
│            No                          │
│            ▼                           │
│     3. Return Method Not Found         │
│     error code -32601                  │
└────────────────────────────────────────┘
```

### Plugin Naming Conventions

To avoid collisions, plugins should use namespaced method names:

| Namespace | Example | Purpose |
|-----------|---------|---------|
| `bare.*` | `bare.ping` | Reserved for core methods |
| `vendor.*` | `vendor.acme.tool` | Vendor-specific plugins |
| `user.*` | `user.custom.action` | User-defined plugins |
| `domain.*` | `domain.health.hl7` | Domain-specific plugins |

### Security Considerations

#### Compile-Time Plugins

Since plugins are compiled into the binary:

1. **Same privilege level**: Plugins run with bare-cua's permissions
2. **Memory safety**: Rust's safety guarantees apply
3. **Audit required**: All plugin code must be reviewed
4. **Supply chain**: Plugin dependencies tracked in Cargo.lock

#### Future: External Process Plugins

When implementing external process plugins:

1. **Sandboxing**: Use Landlock (Linux), Seatbelt (macOS), AppContainer (Windows)
2. **Capability drop**: Plugins don't need capture/input capabilities
3. **Resource limits**: cgroups/systemd limits on CPU/memory
4. **Timeout enforcement**: Kill unresponsive plugins
5. **Input validation**: Strict JSON schema validation before passing to plugins

#### Future: WASM Plugins

WASM provides the best isolation story:

1. **Memory sandbox**: Linear memory with bounds checking
2. **Capability model**: WASI capabilities explicit and restricted
3. **Deterministic execution**: No undefined behavior
4. **Near-native speed**: JIT compilation to host architecture

## Consequences

### Positive

1. **Extensibility**: Core remains small, use cases expand via plugins
2. **Community growth**: Third-party plugins can be shared
3. **Version stability**: Core API changes less frequently
4. **Specialization**: Domain experts write domain plugins

### Negative

1. **Ecosystem fragmentation**: Incompatible plugin versions
2. **Quality variance**: Community plugins may be poorly maintained
3. **Security risk**: Malicious or vulnerable plugins
4. **Discovery problem**: Users need to find and evaluate plugins

### Mitigations

- Curated plugin registry with security audits
- Semantic versioning enforcement
- Plugin capability manifest (declares required permissions)
- Automated testing harness for plugin validation

## Future Directions

### Phase 2: External Process Plugins (6 months)

Spawn plugin as separate process communicating via JSON-RPC:

```rust
// Plugin manifest (plugin.json)
{
    "name": "ml-inference",
    "version": "1.0.0",
    "methods": ["ml.classify", "ml.detect"],
    "command": ["python", "-m", "ml_plugin"],
    "sandbox": {
        "network": false,
        "filesystem": ["/tmp", "/models"],
        "capabilities": []
    }
}
```

### Phase 3: WASM Plugins (12 months)

Compile plugins to WASM for sandboxed execution:

```rust
// Plugin compiled to .wasm
let module = wasmtime::Module::from_file(engine, "plugin.wasm")?;
let instance = linker.instantiate(&mut store, &module)?;

// Call plugin method
let handle = instance.get_typed_func::<(String,), String>(&mut store, "handle")?;
let result = handle.call(&mut store, (params,))?;
```

## Related Decisions

- ADR-001: Hexagonal Architecture with JSON-RPC 2.0 IPC
- ADR-002: Platform Adapter Selection Strategy

## References

- [WebAssembly System Interface (WASI)](https://wasi.dev/)
- [wasmtime](https://wasmtime.dev/) - WASM runtime for Rust
- [Landlock LSM](https://landlock.io/) - Linux sandboxing
- [OpenRPC](https://spec.open-rpc.org/) - API specification

## Traceability

- `@trace BCUA-ARCH-003`
- `@trace BCUA-EXT-001`
