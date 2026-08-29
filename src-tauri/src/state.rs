//! Shared application state. Built once in `bootstrap`, read by every command.
//!
//! Use cases are constructed here and stored behind `Arc`, so a command never resolves a
//! dependency graph on the hot path. See `docs/02-arquitectura.md` §8.

use eo_application::config::AppConfig;
use eo_application::ports::settings::SettingsStore;
use eo_domain::clock::{Clock, SystemClock};
use eo_domain::ids::{IdGenerator, UuidV7Generator};
use eo_infrastructure::paths::AppPaths;
use std::sync::Arc;

pub struct AppState {
    pub paths: AppPaths,
    pub settings: Arc<dyn SettingsStore>,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
}

impl AppState {
    pub fn new(paths: AppPaths, settings: Arc<dyn SettingsStore>) -> Self {
        Self {
            paths,
            settings,
            clock: Arc::new(SystemClock),
            ids: Arc::new(UuidV7Generator),
        }
    }

    pub fn config(&self) -> AppConfig {
        self.settings.snapshot()
    }
}
