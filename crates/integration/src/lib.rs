use pheno_errors::AppError;
use serde::Deserialize;

/// Runtime configuration for the PlayCua application.
///
/// Loaded via `pheno-config` from environment variables with the `PLAYCUA` prefix.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PlayCuaConfig {
    /// Service base URL.
    pub url: String,
    /// Service listen port.
    pub port: u16,
    /// Tracing/log filter level.
    pub log_level: String,
    /// On-disk database path.
    pub db_path: String,
    /// Optional feature flags.
    #[serde(default)]
    pub feature_flags: Vec<String>,
}

/// The canonical PlayCua application harness.
///
/// Wires together the three pheno-* foundation crates:
/// - `pheno-config` for typed configuration loading
/// - `pheno-tracing` for structured logging initialization
/// - `pheno-errors` for canonical error handling
#[derive(Debug, Clone)]
pub struct PlayCuaApp {
    /// The loaded runtime configuration.
    pub config: PlayCuaConfig,
}

impl PlayCuaApp {
    /// Creates a new `PlayCuaApp` by loading configuration from environment
    /// variables with the `PLAYCUA` prefix and initializing the global
    /// tracing subscriber.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Domain` when config loading fails, or
    /// `AppError::Storage` for I/O-related failures.
    pub fn new() -> Result<Self, AppError> {
        let config: PlayCuaConfig =
            pheno_config::load_from_env("PLAYCUA").map_err(|e| AppError::domain(e.to_string()))?;

        pheno_tracing::init();

        tracing::info!(
            url = %config.url,
            port = config.port,
            log_level = %config.log_level,
            db_path = %config.db_path,
            "PlayCuaApp initialized"
        );

        Ok(Self { config })
    }

    /// Creates a `PlayCuaApp` from an explicitly provided config.
    ///
    /// Tracing is still initialized via `pheno-tracing::init()`.
    pub fn with_config(config: PlayCuaConfig) -> Self {
        pheno_tracing::init();

        tracing::info!(
            url = %config.url,
            port = config.port,
            log_level = %config.log_level,
            db_path = %config.db_path,
            "PlayCuaApp initialized with explicit config"
        );

        Self { config }
    }

    /// Placeholder run method — a no-op that returns `Ok(())`.
    ///
    /// Future iterations will wire the actual PlayCua event loop here.
    pub fn run(&self) -> Result<(), AppError> {
        tracing::info!("PlayCuaApp::run() called (no-op placeholder)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_loads_config() {
        // Set env vars so pheno-config can materialise a PlayCuaConfig.
        std::env::set_var("PLAYCUA_URL", "http://localhost:8080");
        std::env::set_var("PLAYCUA_PORT", "8080");
        std::env::set_var("PLAYCUA_LOG_LEVEL", "info");
        std::env::set_var("PLAYCUA_DB_PATH", "/tmp/playcua.db");
        std::env::set_var("PLAYCUA_FEATURE_FLAGS", "[\"beta\",\"gamma\"]");

        let app = PlayCuaApp::new().expect("app should load config from env");
        assert_eq!(app.config.url, "http://localhost:8080");
        assert_eq!(app.config.port, 8080);
        assert_eq!(app.config.log_level, "info");
        assert_eq!(app.config.db_path, "/tmp/playcua.db");
        assert_eq!(app.config.feature_flags, vec!["beta", "gamma"]);
    }

    #[test]
    fn app_initializes_tracing() {
        // Tracing is already global; just verify with_config does not panic
        // and that the app struct is correctly formed.
        let config = PlayCuaConfig {
            url: "http://localhost:9090".to_string(),
            port: 9090,
            log_level: "debug".to_string(),
            db_path: "/var/lib/playcua.db".to_string(),
            feature_flags: vec!["alpha".to_string()],
        };

        let app = PlayCuaApp::with_config(config.clone());
        assert_eq!(app.config, config);
    }

    #[test]
    fn app_run_returns_ok() {
        let config = PlayCuaConfig {
            url: "http://localhost:9090".to_string(),
            port: 9090,
            log_level: "debug".to_string(),
            db_path: "/var/lib/playcua.db".to_string(),
            feature_flags: vec![],
        };

        let app = PlayCuaApp::with_config(config);
        assert!(app.run().is_ok());
    }
}
