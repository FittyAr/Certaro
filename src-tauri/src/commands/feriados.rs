//! Commands of `feriados`. See `docs/11-contratos-tauri.md` §5.13.
//!
//! Every write returns the year's calendar, so the settings screen never has to chain a second
//! call to refresh its list.

use chrono::NaiveDate;
use certaro_application::dtos::feriados::{FeriadoDto, FeriadoInput, FeriadoSyncResult};
use tauri::State;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn feriados_list(state: State<'_, AppState>, anio: i32) -> ApiResult<Vec<FeriadoDto>> {
    let outcome = match state.services() {
        Ok(services) => services.feriados.list(anio).await,
        Err(e) => Err(e),
    };
    handle("feriados_list", outcome)
}

#[tauri::command]
pub async fn feriados_sync(
    state: State<'_, AppState>,
    anios: Vec<i32>,
) -> ApiResult<FeriadoSyncResult> {
    let outcome = match state.services() {
        Ok(services) => services.feriados.sync(anios).await,
        Err(e) => Err(e),
    };
    handle("feriados_sync", outcome)
}

#[tauri::command]
pub async fn feriados_add(
    state: State<'_, AppState>,
    dto: FeriadoInput,
) -> ApiResult<Vec<FeriadoDto>> {
    let outcome = match state.services() {
        Ok(services) => services.feriados.add(dto).await,
        Err(e) => Err(e),
    };
    handle("feriados_add", outcome)
}

#[tauri::command]
pub async fn feriados_delete(
    state: State<'_, AppState>,
    fecha: NaiveDate,
) -> ApiResult<Vec<FeriadoDto>> {
    let outcome = match state.services() {
        Ok(services) => services.feriados.delete(fecha).await,
        Err(e) => Err(e),
    };
    handle("feriados_delete", outcome)
}
