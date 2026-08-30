//! Commands of configuration, backups and the JSON dump. See `docs/11-contratos-tauri.md` §5.13.
//!
//! `backup_restore` and `backup_import_json` are destructive. The frontend confirms them twice and
//! the backend takes a backup of the current state before either runs, so there is always a way
//! back from a mistaken click.

use eo_application::config::AppConfig;
use eo_application::dtos::dashboard::EstadoSistema;
use eo_application::ports::{BackupItem, ImportResumen, VerificacionBackup};
use eo_application::use_cases::configuracion::Cambios;
use tauri::State;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn config_get_all(state: State<'_, AppState>) -> ApiResult<AppConfig> {
    handle("config_get_all", Ok(state.config()))
}

#[tauri::command]
pub async fn config_set(state: State<'_, AppState>, cambios: Cambios) -> ApiResult<AppConfig> {
    let outcome = match state.services() {
        Ok(services) => services.configuracion.set(cambios).await,
        Err(e) => Err(e),
    };
    handle("config_set", outcome)
}

#[tauri::command]
pub async fn config_reset(
    state: State<'_, AppState>,
    claves: Vec<String>,
) -> ApiResult<AppConfig> {
    let outcome = match state.services() {
        Ok(services) => services.configuracion.reset(claves).await,
        Err(e) => Err(e),
    };
    handle("config_reset", outcome)
}

#[tauri::command]
pub async fn sistema_info(state: State<'_, AppState>) -> ApiResult<EstadoSistema> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.info(env!("CARGO_PKG_VERSION")).await,
        Err(e) => Err(e),
    };
    handle("sistema_info", outcome)
}

#[tauri::command]
pub async fn backup_list(state: State<'_, AppState>) -> ApiResult<Vec<BackupItem>> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.backups().await,
        Err(e) => Err(e),
    };
    handle("backup_list", outcome)
}

#[tauri::command]
pub async fn backup_create(state: State<'_, AppState>) -> ApiResult<BackupItem> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.backup_create().await,
        Err(e) => Err(e),
    };
    handle("backup_create", outcome)
}

#[tauri::command]
pub async fn backup_verify(
    state: State<'_, AppState>,
    nombre: String,
) -> ApiResult<VerificacionBackup> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.backup_verify(&nombre).await,
        Err(e) => Err(e),
    };
    handle("backup_verify", outcome)
}

#[tauri::command]
pub async fn backup_restore(state: State<'_, AppState>, nombre: String) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.backup_restore(&nombre).await,
        Err(e) => Err(e),
    };
    handle("backup_restore", outcome)
}

#[tauri::command]
pub async fn backup_export_json(
    state: State<'_, AppState>,
    destino: String,
) -> ApiResult<ImportResumen> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.export_json(&destino).await,
        Err(e) => Err(e),
    };
    handle("backup_export_json", outcome)
}

#[tauri::command]
pub async fn backup_import_json(
    state: State<'_, AppState>,
    origen: String,
) -> ApiResult<ImportResumen> {
    let outcome = match state.services() {
        Ok(services) => services.sistema.import_json(&origen).await,
        Err(e) => Err(e),
    };
    handle("backup_import_json", outcome)
}
