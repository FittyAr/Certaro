//! Loading and persisting configuration. See `docs/14-configuracion-e-i18n.md` §1.
//!
//! Three layers, in increasing precedence: compiled defaults, the user's `config.json`, and
//! `EO_<SECTION>__<KEY>` environment variables. Layer one is always complete, so the application
//! starts with no file at all.

use anyhow::Context;
use certaro_application::config::AppConfig;
use certaro_application::ports::settings::SettingsStore;
use certaro_application::{AppError, AppResult};
use figment::providers::{Env, Format, Json, Serialized};
use figment::Figment;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Reads the three layers. A corrupt `config.json` is renamed and the defaults are used, because
/// refusing to start over an unreadable preferences file is worse than losing the preferences.
/// The legacy system threw while deserialising and the application simply did not open.
pub fn load(config_path: &Path, base: AppConfig) -> anyhow::Result<AppConfig> {
    let mut figment = Figment::from(Serialized::defaults(base.clone()));

    if config_path.exists() {
        match Figment::from(Serialized::defaults(base.clone()))
            .merge(Json::file(config_path))
            .extract::<AppConfig>()
        {
            Ok(_) => figment = figment.merge(Json::file(config_path)),
            Err(e) => {
                let backup = config_path.with_extension("json.bak");
                tracing::warn!(
                    error = %e,
                    path = %config_path.display(),
                    backup = %backup.display(),
                    "unreadable config file, falling back to defaults"
                );
                let _ = std::fs::rename(config_path, &backup);
            }
        }
    }

    let config: AppConfig = figment
        .merge(Env::prefixed("EO_").split("__"))
        .extract()
        .context("merging configuration layers")?;

    Ok(config)
}

/// Writes only what differs from the default, so `config.json` stays short and readable.
pub fn save(config_path: &Path, config: &AppConfig) -> anyhow::Result<()> {
    let full = serde_json::to_value(config).context("serialising configuration")?;
    let defaults = serde_json::to_value(AppConfig::default()).context("serialising defaults")?;
    let diff = prune_defaults(&full, &defaults)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).context("creating the configuration directory")?;
    }
    // Written to a temporary file and renamed so a crash mid-write cannot leave a truncated file
    // that the next start would have to discard.
    let tmp = config_path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(&diff)?).context("writing configuration")?;
    std::fs::rename(&tmp, config_path).context("replacing the configuration file")?;
    Ok(())
}

/// Recursively removes the entries of `value` that are identical to `defaults`.
/// Returns `None` when nothing is left, so empty sections disappear too.
fn prune_defaults(
    value: &serde_json::Value,
    defaults: &serde_json::Value,
) -> Option<serde_json::Value> {
    match (value, defaults) {
        (serde_json::Value::Object(v), serde_json::Value::Object(d)) => {
            let mut out = serde_json::Map::new();
            for (key, val) in v {
                match d.get(key) {
                    Some(def) => {
                        if let Some(pruned) = prune_defaults(val, def) {
                            out.insert(key.clone(), pruned);
                        }
                    }
                    None => {
                        out.insert(key.clone(), val.clone());
                    }
                }
            }
            (!out.is_empty()).then_some(serde_json::Value::Object(out))
        }
        _ if value == defaults => None,
        _ => Some(value.clone()),
    }
}

/// The shared, in-memory configuration. Reads take a snapshot so they never block on a writer.
#[derive(Debug)]
pub struct FileSettingsStore {
    path: PathBuf,
    current: RwLock<AppConfig>,
}

impl FileSettingsStore {
    pub fn new(path: impl Into<PathBuf>, initial: AppConfig) -> Self {
        Self {
            path: path.into(),
            current: RwLock::new(initial),
        }
    }

    #[must_use]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[async_trait::async_trait]
impl SettingsStore for FileSettingsStore {
    fn snapshot(&self) -> AppConfig {
        // A poisoned lock means a writer panicked; the last good snapshot is still readable and
        // more useful than propagating the panic to every reader.
        match self.current.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    async fn save(&self, config: AppConfig) -> AppResult<()> {
        config.validate()?;
        save(&self.path, &config).map_err(AppError::io)?;
        match self.current.write() {
            Ok(mut guard) => *guard = config,
            Err(poisoned) => *poisoned.into_inner() = config,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use certaro_application::config::ThemePreference;

    #[test]
    fn defaults_alone_produce_a_valid_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = load(&dir.path().join("config.json"), AppConfig::default()).unwrap();
        assert_eq!(cfg, AppConfig::default());
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn only_the_changed_keys_are_persisted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut cfg = AppConfig::default();
        cfg.application.theme = ThemePreference::Dark;

        save(&path, &cfg).unwrap();
        let written: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

        assert_eq!(written["application"]["theme"], "dark");
        assert!(
            written.get("locale").is_none(),
            "untouched sections are omitted"
        );
    }

    #[test]
    fn a_corrupt_file_is_renamed_and_does_not_stop_the_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "{ this is not json").unwrap();

        let cfg = load(&path, AppConfig::default()).unwrap();

        assert_eq!(cfg, AppConfig::default());
        assert!(dir.path().join("config.json.bak").exists());
    }

    #[test]
    fn a_saved_value_survives_a_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut cfg = AppConfig::default();
        cfg.business.contratista = "Pablo Baez".to_owned();

        save(&path, &cfg).unwrap();
        let reloaded = load(&path, AppConfig::default()).unwrap();

        assert_eq!(reloaded.business.contratista, "Pablo Baez");
    }
}
