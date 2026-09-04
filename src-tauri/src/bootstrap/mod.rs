use std::sync::Arc;
use tauri::{Emitter, Manager};

use certaro_infrastructure::backup::SqliteBackupService;
use certaro_infrastructure::external::dolar::HttpExchangeRateProvider;
use certaro_infrastructure::external::holidays::HttpHolidayProvider;
use certaro_infrastructure::files::{FsAttachmentStore, SystemOpener};

use crate::state::AppState;
use crate::{EVENT_COTIZACIONES, EVENT_FATAL, EVENT_MANTENIMIENTO, EVENT_READY};

/// Connects the database, applies migrations and seeds the system rows, then publishes the use
/// cases so the commands can serve requests.
pub async fn bootstrap(handle: &tauri::AppHandle) -> anyhow::Result<()> {
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

pub fn spawn_bootstrap(handle: tauri::AppHandle) {
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
}
