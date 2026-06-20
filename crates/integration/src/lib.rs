use configra_config::ConfigraConfig;
use pheno_config::Config;
use pheno_errors::AppError;

/// The canonical PlayCua application harness.
///
/// Wires together the three pheno-* foundation crates plus the Configra
/// substrate per ADR-031:
///
/// - `configra-config` (from `KooshaPari/Configra`) for substrate-level
///   defaults (default port, default log level, db path template,
///   idempotency TTLs, watcher intervals). Loaded from `CONFIGRA_*`
///   environment variables when available, otherwise from hardcoded
///   defaults documented in the Configra workspace.
/// - `pheno-config` for app-specific runtime config (URL, DB_PATH,
///   feature flags). Loaded from `PLAYCUA_*` env vars. The crate
///   itself is now published from the Configra workspace per ADR-031,
///   so this dependency points at `Configra/crates/pheno-config`.
/// - `pheno-tracing` for structured logging initialization.
/// - `pheno-errors` for canonical error handling.
#[derive(Debug, Clone)]
pub struct PlayCuaApp {
    /// The loaded app-specific runtime configuration
    /// (URL, port, log level, db_path, feature flags).
    pub config: Config,
    /// The Configra substrate defaults — port/log-level/db-template
    /// source-of-truth, consumed at construction time.
    pub substrate: ConfigraConfig,
}

impl PlayCuaApp {
    /// Creates a new `PlayCuaApp` by loading configuration from environment
    /// variables (`CONFIGRA_*` for substrate defaults, `PLAYCUA_*` for
    /// app-specific runtime values) and initializing the global tracing
    /// subscriber.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Domain` when config loading fails, or
    /// `AppError::Storage` for I/O-related failures.
    pub fn new() -> Result<Self, AppError> {
        // Substrate defaults from Configra (the ADR-031 canonical name).
        // `from_env` reads `CONFIGRA_*` vars; missing/invalid vars fall
        // back to the documented defaults (never fails — see
        // `configra_config::ConfigraConfig::from_env`).
        let substrate = ConfigraConfig::from_env();

        // App-specific runtime config (URL, DB_PATH, feature flags)
        // still flows through `pheno_config` — the app-level concerns
        // that Configra doesn't model.
        let config =
            pheno_config::load_from_env("PLAYCUA").map_err(|e| AppError::domain(e.to_string()))?;

        pheno_tracing::init();

        tracing::info!(
            url = %config.url,
            port = config.port,
            log_level = %config.log_level,
            db_path = %config.db_path,
            substrate_default_port = substrate.service.default_port,
            substrate_default_log_level = %substrate.service.default_log_level,
            "PlayCuaApp initialized"
        );

        Ok(Self { config, substrate })
    }

    /// Creates a `PlayCuaApp` from an explicitly provided config.
    ///
    /// Substrate defaults come from `ConfigraConfig::default()` so that
    /// callers can introspect the substrate settings without going
    /// through the env. Tracing is still initialized via
    /// `pheno-tracing::init()`.
    pub fn with_config(config: Config) -> Self {
        let substrate = ConfigraConfig::default();

        pheno_tracing::init();

        tracing::info!(
            url = %config.url,
            port = config.port,
            log_level = %config.log_level,
            db_path = %config.db_path,
            substrate_default_port = substrate.service.default_port,
            substrate_default_log_level = %substrate.service.default_log_level,
            "PlayCuaApp initialized with explicit config"
        );

        Self { config, substrate }
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
    use pheno_config::ConfigBuilder;

    /// Helper: build a canonical Config suitable for unit tests
    /// without touching the environment.
    fn sample_config() -> Config {
        ConfigBuilder::new()
            .url("http://localhost:9090")
            .db_path("/var/lib/playcua.db")
            .port(9090)
            .log_level("debug")
            .feature_flag("alpha")
            .build()
            .expect("config should build")
    }

    #[test]
    fn app_loads_config() {
        // Set env vars so pheno-config can materialise a Config.
        std::env::set_var("PLAYCUA_URL", "http://localhost:8080");
        std::env::set_var("PLAYCUA_PORT", "8080");
        std::env::set_var("PLAYCUA_LOG_LEVEL", "info");
        std::env::set_var("PLAYCUA_DB_PATH", "/tmp/playcua.db");
        std::env::set_var("PLAYCUA_FEATURE_FLAGS", "beta,gamma");

        let app = PlayCuaApp::new().expect("app should load config from env");
        assert_eq!(app.config.url, "http://localhost:8080");
        assert_eq!(app.config.port, 8080);
        assert_eq!(app.config.log_level, "info");
        assert_eq!(app.config.db_path, "/tmp/playcua.db");
        assert_eq!(
            app.config.feature_flags,
            vec!["beta".to_string(), "gamma".to_string()]
        );

        // The Configra substrate defaults are now visible on the app
        // struct (ADR-031 migration verification).
        assert_eq!(app.substrate.service.default_port, 8080);
        assert_eq!(app.substrate.service.default_log_level, "info");
    }

    #[test]
    fn app_initializes_tracing() {
        // Tracing is global and idempotent — with_config must not panic
        // and the resulting app must round-trip the config it was given.
        let config = sample_config();

        let app = PlayCuaApp::with_config(config.clone());
        assert_eq!(app.config, config);

        // Substrate defaults are sourced from `ConfigraConfig::default()`
        // in `with_config` — verify they match the documented values.
        assert_eq!(app.substrate.service.default_port, 8080);
        assert_eq!(app.substrate.service.default_log_level, "info");
        assert_eq!(
            app.substrate.service.db_path_template,
            "/var/lib/{name}.db"
        );
    }

    #[test]
    fn app_run_returns_ok() {
        let config = sample_config();
        let app = PlayCuaApp::with_config(config);
        assert!(app.run().is_ok());
    }

    /// Migration test (ADR-031): verify that `ConfigraConfig::default()`
    /// is reachable through `PlayCuaApp::with_config` and that its
    /// documented defaults are exposed on the app struct.
    #[test]
    fn substrate_defaults_visible_on_app() {
        let app = PlayCuaApp::with_config(sample_config());

        // Service defaults
        assert_eq!(app.substrate.service.default_port, 8080);
        assert_eq!(app.substrate.service.default_log_level, "info");
        assert_eq!(
            app.substrate.service.db_path_template,
            "/var/lib/{name}.db"
        );

        // Idempotency defaults
        assert_eq!(app.substrate.idempotency.default_ttl_secs, 86_400);
        assert_eq!(app.substrate.idempotency.default_max_retries, 3);

        // Watcher defaults
        assert_eq!(app.substrate.watcher.poll_interval_ms, 1000);
        assert!(app.substrate.watcher.enabled);
    }
}
