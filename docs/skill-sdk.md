# Skill SDK — Authoring third-party JSON-RPC methods

The **Skill SDK** is the recommended way to extend `playcua-native` with
additional JSON-RPC methods without forking the binary or modifying the
core dispatcher. A *skill* is a Rust type that implements
[`MethodPlugin`](../../native/src/plugins/mod.rs) and registers itself
with the global `PluginRegistry`.

The dispatcher consults the plugin registry after the built-in method
handlers fail to match, so a plugin's method name simply needs to not
collide with a built-in (`screenshot`, `input.*`, `windows.*`,
`process.*`, `analysis.*`, `ping`). Convention: use a dotted
`<namespace>.<verb>` form (e.g. `acme.tts`, `sentry.annotate`).

## When to use a skill

| Need | Use a skill? |
|---|---|
| Add a new JSON-RPC method that takes params and returns JSON | yes |
| Add a new modality (capture/input/process adapter) | no — add a new adapter in `adapters/` instead |
| Add a new transport (gRPC, websockets, etc.) | no — add a new transport in `transports/` |
| Replace an existing built-in method | no — fork the dispatcher |

## The `MethodPlugin` trait

```rust
#[async_trait::async_trait]
pub trait MethodPlugin: Send + Sync {
    /// The exact method name this plugin handles (e.g. "custom.foo").
    fn method_name(&self) -> &'static str;

    /// Handle an incoming request. `params` is the raw JSON params value
    /// (may be Null if no params were provided).
    async fn handle(&self, params: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}
```

The two methods have to return `'static` for the method name and
`Send + Sync` for the trait object — this is what makes the registry
safe to share across tokio tasks (one per connection in daemon mode).

## Walkthrough: an "echo" skill

This is the canonical example, also used in the unit tests in
[`native/src/plugins/mod.rs`](../../native/src/plugins/mod.rs).

### 1. Create a new crate

```bash
mkdir acme-skills && cd acme-skills
cargo init --lib --name acme_skills
cargo add playcua-native --path ../native  # adjust path
cargo add async-trait serde_json
```

### 2. Implement the plugin

```rust
// src/lib.rs
use async_trait::async_trait;
use playcua_native::plugins::MethodPlugin;
use serde_json::Value;

pub struct EchoPlugin;

#[async_trait]
impl MethodPlugin for EchoPlugin {
    fn method_name(&self) -> &'static str { "acme.echo" }

    async fn handle(&self, params: Value) -> anyhow::Result<Value> {
        // Whatever JSON came in, send it back. Useful for ping/keepalive
        // and for protocol-level smoke tests.
        Ok(params)
    }
}
```

### 3. Register at startup

In your main binary:

```rust
use playcua_native::plugins::PluginRegistry;
use acme_skills::EchoPlugin;

let mut registry = PluginRegistry::new();
registry.register(Box::new(EchoPlugin));
// … pass `registry` to your dispatcher wiring
```

### 4. Call from a client

```bash
$ playcua-cli
> {"jsonrpc":"2.0","id":1,"method":"acme.echo","params":{"msg":"hi"}}
< {"jsonrpc":"2.0","id":1,"result":{"msg":"hi"}}
```

Or via the MCP server (`playcua-mcp` will see `acme_echo` as a tool
whose name is derived from `acme.echo`).

## Receiving state in plugins

A plugin is a regular `Send + Sync` type — it can hold `Arc<T>` for any
shared state. For example, a `BrowserSession` plugin might hold an
`Arc<Mutex<Browser>>` to keep a single browser instance across calls.

```rust
pub struct ScreenshotAnnotatePlugin {
    ann: Arc<AnnotationStore>,
}

impl ScreenshotAnnotatePlugin {
    pub fn new() -> Self {
        Self { ann: Arc::new(AnnotationStore::default()) }
    }
}

#[async_trait]
impl MethodPlugin for ScreenshotAnnotatePlugin {
    fn method_name(&self) -> &'static str { "sentry.annotate" }

    async fn handle(&self, params: Value) -> anyhow::Result<Value> {
        // Sentry-style flow: capture, annotate, store.
        let path: String = serde_json::from_value(params["path"].clone())?;
        let mut ann = self.ann.lock().await;
        ann.tag(&path, params["tag"].as_str().unwrap_or("untriaged"));
        Ok(serde_json::json!({ "tagged": true, "path": path }))
    }
}
```

## Testing

Test your plugin directly against the trait, no daemon required:

```rust
#[tokio::test]
async fn echo_round_trip() {
    let plugin = EchoPlugin;
    let out = plugin.handle(serde_json::json!({ "x": 1 })).await.unwrap();
    assert_eq!(out, serde_json::json!({ "x": 1 }));
}
```

The `PluginRegistry` also has a `find()` method you can use to verify
the plugin registered itself correctly.

## Distribution

Three patterns, in order of integration depth:

1. **Built into `playcua-native`** — add your plugin to
   `native/src/bin/playcua-native.rs` after the built-in plugin
   registrations. Easiest, but ties your release to the daemon.
2. **Loaded from a workspace member** — if you have multiple skills
   in the same cargo workspace as `playcua-native`, add a new binary
   that wires your plugins in and re-exports the daemon. Cleanest
   for the monorepo case.
3. **Dynamically loaded from `$PLAYCUA_PLUGIN_DIR`** — *not yet
   implemented* (tracked in ADR-006). Will require a stable ABI
   for the trait; expect a `cdylib` + `libloading` design.

For local development, option 2 is the recommended path.

## Security

A plugin runs in the same process as the daemon, with the same
filesystem and network permissions. Treat plugin code as trusted:

- A malicious plugin can read any file the daemon can read
- A plugin that calls `process::launch` from the dispatcher ports
  inherits the dispatcher's process-launch permissions
- Plugins **cannot** be loaded over the network in this slice
- Plugin errors return JSON-RPC `-32603 INTERNAL_ERROR` to the
  client; they do not crash the daemon

For untrusted plugins, run the daemon inside a modality
(`--modality sandbox` or `--modality container`) — the modality layer
sandbox boundaries will then constrain the plugin's reach.

## Reference

- Trait: [`native/src/plugins/mod.rs`](../../native/src/plugins/mod.rs)
- Registry: `PluginRegistry::register` / `PluginRegistry::find`
- Test examples: same file, `mod tests`
- MCP bridge: `playcua-mcp` exposes registered plugin methods as MCP
  tools automatically (name conversion: `acme.echo` → `acme_echo`).
