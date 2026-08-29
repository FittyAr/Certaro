//! Tauri adapter: wiring, state and commands. No SQL and no business rules.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::panic)]

pub mod commands;
pub mod error;
pub mod state;

use eo_application::config::{AppConfig, Environment};
use eo_infrastructure::paths::AppPaths;
use eo_infrastructure::{config as infra_config, telemetry};
use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};

/// Emitted once the background bootstrap succeeded. The interface shows an "initialising" state
/// until it arrives.
pub const EVENT_READY: &str = "app://ready";
/// Emitted when bootstrap failed. Carries the i18n key, never a raw message.
pub const EVENT_FATAL: &str = "app://fatal";

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
            tauri::async_runtime::spawn(async move {
                match bootstrap(&handle).await {
                    Ok(()) => {
                        tracing::info!("bootstrap complete");
                        let _ = handle.emit(EVENT_READY, ());
                    }
                    Err(e) => {
                        tracing::error!(cause = ?e, "bootstrap failed");
                        let _ = handle.emit(EVENT_FATAL, "Error.BootstrapFailed");
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::ping,
            commands::app::app_info,
            commands::app::app_config,
            commands::tipos_movimiento::tipos_movimiento_list,
            commands::tipos_movimiento::tipos_movimiento_get,
            commands::tipos_movimiento::tipos_movimiento_create,
            commands::tipos_movimiento::tipos_movimiento_update,
            commands::tipos_movimiento::tipos_movimiento_delete,
            commands::tipos_movimiento::tipos_movimiento_lookup,
            commands::categorias::categorias_list,
            commands::categorias::categorias_get,
            commands::categorias::categorias_create,
            commands::categorias::categorias_update,
            commands::categorias::categorias_delete,
            commands::categorias::categorias_lookup,
            commands::movimientos::movimientos_list,
            commands::movimientos::movimientos_get,
            commands::movimientos::movimientos_resumen,
            commands::movimientos::movimientos_create,
            commands::movimientos::movimientos_update,
            commands::movimientos::movimientos_delete,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            tracing::error!(cause = ?e, "the application could not start");
        });
}

/// Connects the database, applies migrations and seeds the system rows, then publishes the use
/// cases so the commands can serve requests.
async fn bootstrap(handle: &tauri::AppHandle) -> anyhow::Result<()> {
    let state = handle.state::<AppState>();
    let db = eo_infrastructure::persistence::open(&state.paths.database()).await?;
    state.install_services(Arc::new(
        eo_infrastructure::persistence::SeaOrmUnitOfWork::new(db),
    ));
    Ok(())
}

/// Convenience for tests and for the `dev` profile check.
#[must_use]
pub const fn is_development(config: &AppConfig) -> bool {
    matches!(config.application.environment, Environment::Development)
}
