//! Plugin system — extensible method dispatch via registered MethodPlugin impls.
//!
//! Plugins allow third-party code to register additional JSON-RPC methods
//! without modifying the core dispatcher. The PluginRegistry is consulted
//! after built-in methods fail to match in the dispatcher.

use async_trait::async_trait;
use serde_json::Value;

/// A plugin that handles a single JSON-RPC method name.
#[async_trait]
#[allow(dead_code)]
pub trait MethodPlugin: Send + Sync {
    /// The exact method name this plugin handles (e.g. "custom.foo").
    fn method_name(&self) -> &'static str;

    /// Handle an incoming request. `params` is the raw JSON params value
    /// (may be Null if no params were provided).
    async fn handle(&self, params: Value) -> anyhow::Result<Value>;
}

/// Registry of all registered plugins, keyed by method name.
#[allow(dead_code)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn MethodPlugin>>,
}

impl PluginRegistry {
    /// Create an empty registry.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// Register a plugin. If a plugin with the same method name is already
    /// registered, the new one replaces it.
    #[allow(dead_code)]
    pub fn register(&mut self, plugin: Box<dyn MethodPlugin>) {
        if let Some(pos) = self.plugins.iter().position(|p| p.method_name() == plugin.method_name()) {
            self.plugins[pos] = plugin;
        } else {
            self.plugins.push(plugin);
        }
    }

    /// Find the plugin registered for `method`, if any.
    #[allow(dead_code)]
    pub fn find(&self, method: &str) -> Option<&dyn MethodPlugin> {
        self.plugins
            .iter()
            .find(|p| p.method_name() == method)
            .map(|p| p.as_ref())
    }

    /// Returns the number of registered plugins.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns true if no plugins are registered.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct EchoPlugin;

    #[async_trait]
    impl MethodPlugin for EchoPlugin {
        fn method_name(&self) -> &'static str {
            "test.echo"
        }
        async fn handle(&self, params: Value) -> anyhow::Result<Value> {
            Ok(params)
        }
    }

    #[tokio::test]
    async fn test_register_and_find() {
        let mut registry = PluginRegistry::new();
        assert!(registry.find("test.echo").is_none());
        registry.register(Box::new(EchoPlugin));
        let plugin = registry.find("test.echo").expect("plugin should be registered");
        let result = plugin.handle(json!({ "msg": "hello" })).await.unwrap();
        assert_eq!(result, json!({ "msg": "hello" }));
    }

    #[tokio::test]
    async fn test_replace_on_duplicate_register() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(EchoPlugin));
        registry.register(Box::new(EchoPlugin)); // replace
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_find_missing_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.find("not.registered").is_none());
    }
}
