//! Tauri adapter: wiring, state and commands. No SQL and no business rules.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::panic)]

pub mod bootstrap;
pub mod commands;
pub mod error;
pub mod state;

use certaro_application::config::{AppConfig, Environment};
use certaro_infrastructure::paths::AppPaths;
use certaro_infrastructure::{config as infra_config, telemetry};
use state::AppState;
use std::sync::Arc;
use tauri::Manager;

/// Emitted once the background bootstrap succeeded. The interface shows an "initialising" state
/// until it arrives.
pub const EVENT_READY: &str = "app://ready";
/// Emitted when bootstrap failed. Carries the i18n key, never a raw message.
pub const EVENT_FATAL: &str = "app://fatal";
/// Emitted when the dollar quotes were refreshed in the background, so the status bar updates
/// without polling. See `docs/11-contratos-tauri.md` §6.
pub const EVENT_COTIZACIONES: &str = "cotizaciones:updated";
/// Emitted once the startup housekeeping finished, so the system section can show what it did.
pub const EVENT_MANTENIMIENTO: &str = "mantenimiento:done";

pub fn run() {
    // Configuration and logging come up before the window, so a failure in either is on disk.
    let base = if cfg!(debug_assertions) {
        AppConfig::for_development()
    } else {
        AppConfig::default()
    };

    let bootstrap_paths =
        AppPaths::resolve(base.application.data_dir.as_deref(), &base.application.name);
    if let Err(e) = bootstrap_paths.ensure_dirs() {
        eprintln!("cannot create the data directory: {e}");
        return;
    }

    let config = infra_config::load(&bootstrap_paths.config(), base).unwrap_or_else(|e| {
        eprintln!("cannot read configuration: {e}");
        AppConfig::default()
    });

    // The data directory may itself be configured, so it is resolved again with the merged value.
    let paths = AppPaths::resolve(
        config.application.data_dir.as_deref(),
        &config.application.name,
    );
    let _ = paths.ensure_dirs();

    let _telemetry = match telemetry::init(&config.logging, &paths.logs()) {
        Ok(guard) => Some(guard),
        Err(e) => {
            eprintln!("cannot initialise logging: {e}");
            None
        }
    };
    telemetry::prune_old_logs(&paths.logs(), config.logging.retention_days);

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        environment = ?config.application.environment,
        data_dir = %paths.root().display(),
        "starting"
    );

    let settings = Arc::new(infra_config::FileSettingsStore::new(
        paths.config(),
        config.clone(),
    ));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            let handle = app.handle().clone();
            let state = AppState::new(paths.clone(), settings.clone());
            app.manage(state);

            // Bootstrap runs in the background: opening the database, migrating and seeding must
            // not hold the window closed. The legacy application already did this and the pattern
            // is worth keeping.
            bootstrap::spawn_bootstrap(handle);
            Ok(())
        })
        .invoke_handler(commands::generate_handler())
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            tracing::error!(cause = ?e, "the application could not start");
        });
}

/// Convenience for tests and for the `dev` profile check.
#[must_use]
pub const fn is_development(config: &AppConfig) -> bool {
    matches!(config.application.environment, Environment::Development)
}
