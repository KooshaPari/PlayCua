use pheno_config::Config;
use pheno_errors::AppError;
use phenotype_config_loader::{load_json, load_toml};
use std::path::Path;

/// The canonical PlayCua application harness.
///
/// Wires together the three pheno-* foundation crates plus the Configra
/// substrate per ADR-031:
///
/// - `pheno-config` (now in `KooshaPari/Configra` per ADR-031) for
///   app-specific runtime config (URL, DB_PATH, feature flags) and
///   the env-prefix loader (`PLAYCUA_*` vars).
/// - `phenotype-config-loader` (also in `KooshaPari/Configra`,
///   renamed from `configra-config` during the L5-110 substrate
///   audit) for typed JSON/TOML file loading when a config file is
///   supplied. Falls through to env-only when no file is provided.
/// - `pheno-tracing` for structured logging initialization.
/// - `pheno-errors` for canonical error handling.
#[derive(Debug, Clone)]
pub struct PlayCuaApp {
    /// The loaded app-specific runtime configuration
    /// (URL, port, log level, db_path, feature flags).
    pub config: Config,
    /// The path to the config file used to load this app's config,
    /// if any. `None` means env-only loading was used.
    pub config_source: Option<std::path::PathBuf>,
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
        let config =
            pheno_config::load_from_env("PLAYCUA").map_err(|e| AppError::domain(e.to_string()))?;

        pheno_tracing::init();

        tracing::info!(
            url = %config.url,
            port = config.port,
            log_level = %config.log_level,
            db_path = %config.db_path,
            "PlayCuaApp initialized from env"
        );

        Ok(Self {
            config,
            config_source: None,
        })
    }

    /// Creates a `PlayCuaApp` from a JSON or TOML config file using the
    /// Configra `phenotype-config-loader` substrate (ADR-031, L5-110).
    ///
    /// The file extension (`.json` or `.toml`) selects the parser via
    /// `phenotype_config_loader::load_json` / `load_toml`. Other
    /// extensions return `AppError::Domain`.
    ///
    /// Tracing is still initialized via `pheno-tracing::init()`.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Domain` for parse errors or unsupported
    /// extensions; `AppError::Storage` for I/O failures.
    pub fn from_file(path: &Path) -> Result<Self, AppError> {
        let config: Config = match path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("json") => load_json::<Config>(path)
                .map_err(|e| AppError::domain(format!("json load: {e}")))?,
            Some("toml") => load_toml::<Config>(path)
                .map_err(|e| AppError::domain(format!("toml load: {e}")))?,
            Some(other) => {
                return Err(AppError::domain(format!(
                    "unsupported config file extension: `.{other}` (expected `.json` or `.toml`)"
                )));
            }
            None => {
                return Err(AppError::domain(
                    "config file has no extension (expected `.json` or `.toml`)".to_owned(),
                ));
            }
        };

        pheno_tracing::init();

        tracing::info!(
            url = %config.url,
            port = config.port,
            log_level = %config.log_level,
            db_path = %config.db_path,
            config_source = %path.display(),
            "PlayCuaApp initialized from file"
        );

        Ok(Self {
            config,
            config_source: Some(path.to_path_buf()),
        })
    }

    /// Creates a `PlayCuaApp` from an explicitly provided config.
    ///
    /// Tracing is still initialized via `pheno-tracing::init()`.
    pub fn with_config(config: Config) -> Self {
        pheno_tracing::init();

        tracing::info!(
            url = %config.url,
            port = config.port,
            log_level = %config.log_level,
            db_path = %config.db_path,
            "PlayCuaApp initialized with explicit config"
        );

        Self {
            config,
            config_source: None,
        }
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
    use std::io::Write;

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
        // Env-loaded apps have no file source.
        assert!(app.config_source.is_none());
    }

    #[test]
    fn app_initializes_tracing() {
        // Tracing is global and idempotent — with_config must not panic
        // and the resulting app must round-trip the config it was given.
        let config = sample_config();

        let app = PlayCuaApp::with_config(config.clone());
        assert_eq!(app.config, config);
        assert!(app.config_source.is_none());
    }

    #[test]
    fn app_run_returns_ok() {
        let config = sample_config();
        let app = PlayCuaApp::with_config(config);
        assert!(app.run().is_ok());
    }

    /// Migration test (ADR-031, L5-110): verify that the renamed
    /// `phenotype-config-loader` substrate can be used to load a
    /// `Config` from a JSON file on disk.
    #[test]
    fn app_loads_from_json_file() {
        let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
        let path = tmp.path().with_extension("json");
        writeln!(
            tmp.as_file_mut(),
            r#"{{
                "url": "http://localhost:7777",
                "port": 7777,
                "log_level": "warn",
                "db_path": "/var/lib/playcua-file.json.db",
                "feature_flags": ["delta", "epsilon"]
            }}"#
        )
        .expect("write json");
        drop(tmp);

        // We wrote to a path with .json extension. Reload from there.
        let app = PlayCuaApp::from_file(&path).expect("app should load from json file");
        assert_eq!(app.config.url, "http://localhost:7777");
        assert_eq!(app.config.port, 7777);
        assert_eq!(app.config.log_level, "warn");
        assert_eq!(app.config.db_path, "/var/lib/playcua-file.json.db");
        assert_eq!(
            app.config.feature_flags,
            vec!["delta".to_string(), "epsilon".to_string()]
        );
        assert_eq!(app.config_source.as_deref(), Some(path.as_path()));
    }

    /// Migration test (ADR-031, L5-110): verify TOML file loading
    /// via `phenotype-config-loader::load_toml`.
    #[test]
    fn app_loads_from_toml_file() {
        let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
        let path = tmp.path().with_extension("toml");
        writeln!(
            tmp.as_file_mut(),
            r#"
url = "http://localhost:6666"
port = 6666
log_level = "trace"
db_path = "/var/lib/playcua-file.toml.db"
feature_flags = ["zeta"]
"#
        )
        .expect("write toml");
        drop(tmp);

        let app = PlayCuaApp::from_file(&path).expect("app should load from toml file");
        assert_eq!(app.config.url, "http://localhost:6666");
        assert_eq!(app.config.port, 6666);
        assert_eq!(app.config.log_level, "trace");
        assert_eq!(app.config.db_path, "/var/lib/playcua-file.toml.db");
        assert_eq!(app.config.feature_flags, vec!["zeta".to_string()]);
    }

    /// Negative test: unsupported extension should produce a domain
    /// error, not panic.
    #[test]
    fn from_file_unsupported_extension_errors() {
        let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
        let path = tmp.path().with_extension("yaml");
        writeln!(tmp.as_file_mut(), "url: not-relevant").expect("write");
        drop(tmp);

        let result = PlayCuaApp::from_file(&path);
        assert!(result.is_err());
    }
}
