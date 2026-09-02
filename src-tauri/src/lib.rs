//! Tauri adapter: wiring, state and commands. No SQL and no business rules.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::panic)]

pub mod commands;
pub mod error;
pub mod state;

use certaro_application::config::{AppConfig, Environment};
use certaro_infrastructure::backup::SqliteBackupService;
use certaro_infrastructure::external::dolar::HttpExchangeRateProvider;
use certaro_infrastructure::external::holidays::HttpHolidayProvider;
use certaro_infrastructure::files::{FsAttachmentStore, SystemOpener};
use certaro_infrastructure::paths::AppPaths;
use certaro_infrastructure::{config as infra_config, telemetry};
use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};

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
            commands::app::app_is_ready,
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
            commands::clientes::clientes_list,
            commands::clientes::clientes_get,
            commands::clientes::clientes_create,
            commands::clientes::clientes_update,
            commands::clientes::clientes_delete,
            commands::clientes::clientes_lookup,
            commands::proyectos::proyectos_list,
            commands::proyectos::proyectos_get,
            commands::proyectos::proyectos_create,
            commands::proyectos::proyectos_update,
            commands::proyectos::proyectos_transition,
            commands::proyectos::proyectos_delete,
            commands::proyectos::proyectos_lookup,
            commands::proyectos::proyectos_siguiente_numero,
            commands::trabajos::trabajos_list,
            commands::trabajos::trabajos_get,
            commands::trabajos::trabajos_create,
            commands::trabajos::trabajos_update,
            commands::trabajos::trabajos_transition,
            commands::trabajos::trabajos_delete,
            commands::trabajos::trabajos_lookup,
            commands::facturas::facturas_list,
            commands::facturas::facturas_get,
            commands::facturas::facturas_create,
            commands::facturas::facturas_update,
            commands::facturas::facturas_transition,
            commands::facturas::facturas_delete,
            commands::facturas::facturas_lookup,
            commands::facturas::facturas_pagos,
            commands::facturas::facturas_pago_create,
            commands::facturas::facturas_pago_update,
            commands::facturas::facturas_pago_delete,
            commands::ordenes_trabajo::ordenes_trabajo_list,
            commands::ordenes_trabajo::ordenes_trabajo_get,
            commands::ordenes_trabajo::ordenes_trabajo_create,
            commands::ordenes_trabajo::ordenes_trabajo_update,
            commands::ordenes_trabajo::ordenes_trabajo_delete,
            commands::ordenes_trabajo::ordenes_trabajo_lookup,
            commands::certificados::certificados_list,
            commands::certificados::certificados_get,
            commands::certificados::certificados_preparar,
            commands::certificados::certificados_create,
            commands::certificados::certificados_update_observaciones,
            commands::certificados::certificados_delete,
            commands::empleados::empleados_list,
            commands::empleados::empleados_get,
            commands::empleados::empleados_create,
            commands::empleados::empleados_update,
            commands::empleados::empleados_delete,
            commands::empleados::empleados_lookup,
            commands::empleados::empleados_cargos,
            commands::asistencias::asistencia_grilla,
            commands::asistencias::asistencia_upsert,
            commands::asistencias::asistencia_upsert_rango,
            commands::asistencias::asistencia_delete,
            commands::liquidaciones::liquidaciones_list,
            commands::liquidaciones::liquidaciones_get,
            commands::liquidaciones::liquidaciones_suggest,
            commands::liquidaciones::liquidaciones_create,
            commands::liquidaciones::liquidaciones_create_batch,
            commands::liquidaciones::liquidaciones_update,
            commands::liquidaciones::liquidaciones_delete,
            commands::feriados::feriados_list,
            commands::feriados::feriados_sync,
            commands::feriados::feriados_add,
            commands::feriados::feriados_delete,
            commands::dashboard::dashboard_stats,
            commands::dashboard::dashboard_alertas,
            commands::dashboard::cotizaciones_get,
            commands::dashboard::cotizaciones_refresh,
            commands::reportes::movimientos_export,
            commands::reportes::liquidacion_export,
            commands::reportes::certificado_export,
            commands::reportes::reportes_nombre_sugerido,
            commands::comercial::clientes_cuenta_corriente,
            commands::comercial::clientes_antiguedad_deuda,
            commands::comercial::proyectos_rentabilidad,
            commands::comercial::trabajos_rentabilidad,
            commands::adjuntos::adjuntos_list,
            commands::adjuntos::adjuntos_add,
            commands::adjuntos::adjuntos_delete,
            commands::adjuntos::adjuntos_open,
            commands::adjuntos::adjuntos_reveal,
            commands::sistema::config_get_all,
            commands::sistema::config_set,
            commands::sistema::config_reset,
            commands::sistema::sistema_info,
            commands::sistema::backup_list,
            commands::sistema::backup_create,
            commands::sistema::backup_verify,
            commands::sistema::backup_restore,
            commands::sistema::backup_export_json,
            commands::sistema::backup_import_json,
            commands::sistema::sistema_detect_legacy_db,
            commands::sistema::sistema_run_legacy_import,
            commands::sistema::dev_seed_database,
            commands::auth::auth_get_mode,
            commands::auth::auth_current_user,
            commands::auth::auth_login,
            commands::auth::auth_logout,
            commands::auth::auth_configurar_2fa,
            commands::auth::auth_activar_2fa,
            commands::auth::auth_desactivar_2fa,
            commands::auth::usuarios_list,
            commands::auth::usuarios_get,
            commands::auth::usuarios_create,
            commands::auth::usuarios_update,
            commands::auth::usuarios_delete,
            commands::auth::roles_list,
            commands::auth::roles_get,
            commands::auth::roles_create,
            commands::auth::roles_update,
            commands::auth::roles_delete,
            commands::auth::permisos_list,
            commands::kanban::kanban_list_tableros,
            commands::kanban::kanban_get_tablero,
            commands::kanban::kanban_create_tablero,
            commands::kanban::kanban_update_tablero,
            commands::kanban::kanban_delete_tablero,
            commands::kanban::kanban_create_columna,
            commands::kanban::kanban_update_columna,
            commands::kanban::kanban_delete_columna,
            commands::kanban::kanban_create_tarjeta,
            commands::kanban::kanban_update_tarjeta,
            commands::kanban::kanban_mover_tarjeta,
            commands::kanban::kanban_delete_tarjeta,
            commands::kanban::kanban_sincronizar_preset,
            commands::kanban::kanban_list_etiquetas,
            commands::kanban::kanban_create_etiqueta,
            commands::kanban::kanban_update_etiqueta,
            commands::kanban::kanban_delete_etiqueta,
            commands::kanban::kanban_list_checklist,
            commands::kanban::kanban_add_checklist_item,
            commands::kanban::kanban_update_checklist_item,
            commands::kanban::kanban_delete_checklist_item,
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
    let config = state.config();
    let db = certaro_infrastructure::persistence::open_from_config(
        &config.database,
        &state.paths.database(),
    )
    .await?;
    // Behind a handle so that restoring a backup can close the connection before the file under it
    // is replaced. See `docs/13-servicios-externos-y-archivos.md` §4.3.
    let db = certaro_infrastructure::persistence::DbHandle::new(db);
    let holidays = Arc::new(HttpHolidayProvider::new(&config.external_apis)?);
    let dolar = Arc::new(HttpExchangeRateProvider::new(&config.external_apis)?);
    let attachments = Arc::new(FsAttachmentStore::new(
        state.paths.clone(),
        Arc::clone(&state.settings),
        Arc::clone(&state.clock),
    ));
    let backup = Arc::new(SqliteBackupService::new(
        db.clone(),
        state.paths.clone(),
        Arc::clone(&state.settings),
        Arc::clone(&state.clock),
        env!("CARGO_PKG_VERSION"),
    ));
    let hasher = Arc::new(certaro_infrastructure::auth::Argon2PasswordHasher::new());
    let tokens = Arc::new(certaro_infrastructure::auth::Sha256TokenService::new());
    let totp = Arc::new(certaro_infrastructure::auth::TotpService::new());
    state.install_services(
        db.clone(),
        Arc::new(certaro_infrastructure::persistence::SeaOrmUnitOfWork::new(db)),
        holidays,
        dolar,
        attachments,
        Arc::new(SystemOpener),
        backup,
        hasher,
        tokens,
        totp,
    );

    // The calendar is synced here and not on demand: a settlement that cannot see the holidays pays
    // less, so the table is filled before the first wizard runs. A failure only warns.
    if let Ok(services) = state.services() {
        if let Err(e) = services.feriados.sync_al_iniciar().await {
            tracing::warn!(cause = ?e, "the holiday calendar could not be synced at startup");
        }

        // The quotes are warmed here so the status bar has a number to show on the first paint.
        // The service degrades to the cache on its own, so nothing here can fail the bootstrap.
        if config.external_apis.dollar_auto_update {
            match services.cotizaciones.list().await {
                Ok(cotizaciones) if !cotizaciones.is_empty() => {
                    let _ = handle.emit(EVENT_COTIZACIONES, cotizaciones);
                }
                Ok(_) => tracing::info!("no dollar quotes available at startup"),
                Err(e) => tracing::warn!(cause = ?e, "the dollar quotes could not be read"),
            }
        }
    }

    // Housekeeping runs after everything the interface needs is in place, in its own task, so a
    // slow backup never delays `app://ready`. See `docs/13` §6.
    let mantenimiento = handle.clone();
    tauri::async_runtime::spawn(async move {
        let state = mantenimiento.state::<AppState>();
        if let Ok(services) = state.services() {
            let resultado = services.sistema.mantenimiento().await;
            let _ = mantenimiento.emit(EVENT_MANTENIMIENTO, resultado);
        }
    });

    Ok(())
}

/// Convenience for tests and for the `dev` profile check.
#[must_use]
pub const fn is_development(config: &AppConfig) -> bool {
    matches!(config.application.environment, Environment::Development)
}
