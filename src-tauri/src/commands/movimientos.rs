//! Commands of `movimientos`. See `docs/11-contratos-tauri.md` §5.1.

use certaro_application::dtos::common::ListQuery;
use certaro_application::dtos::movimientos::{
    MovimientoDetalle, MovimientoFiltroDto, MovimientoInput, MovimientoListResult,
    MovimientoResumenDto,
};
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn movimientos_list(
    state: State<'_, AppState>,
    query: ListQuery<MovimientoFiltroDto>,
) -> ApiResult<MovimientoListResult> {
    let outcome = match state.services() {
        Ok(services) => services.movimientos.list(query).await,
        Err(e) => Err(e),
    };
    handle("movimientos_list", outcome)
}

#[tauri::command]
pub async fn movimientos_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<MovimientoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.movimientos.get(id).await,
        Err(e) => Err(e),
    };
    handle("movimientos_get", outcome)
}

/// The totals of a filter without its rows, for screens that show the balance and nothing else.
#[tauri::command]
pub async fn movimientos_resumen(
    state: State<'_, AppState>,
    filtro: MovimientoFiltroDto,
) -> ApiResult<MovimientoResumenDto> {
    let outcome = match state.services() {
        Ok(services) => services.movimientos.resumen(filtro).await,
        Err(e) => Err(e),
    };
    handle("movimientos_resumen", outcome)
}

#[tauri::command]
pub async fn movimientos_create(
    state: State<'_, AppState>,
    dto: MovimientoInput,
) -> ApiResult<MovimientoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.movimientos.create(dto).await,
        Err(e) => Err(e),
    };
    handle("movimientos_create", outcome)
}

#[tauri::command]
pub async fn movimientos_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: MovimientoInput,
    row_version: String,
) -> ApiResult<MovimientoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.movimientos.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("movimientos_update", outcome)
}

#[tauri::command]
pub async fn movimientos_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.movimientos.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("movimientos_delete", outcome)
}
