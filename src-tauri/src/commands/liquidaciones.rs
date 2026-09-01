//! Commands of `liquidaciones`. See `docs/11-contratos-tauri.md` §5.8.
//!
//! `liquidaciones_suggest` is pure: it computes and persists nothing, and takes several employees
//! in one call so the wizard does not fire N requests.

use certaro_application::dtos::common::ListQuery;
use certaro_application::dtos::liquidaciones::{
    LiquidacionBatchInput, LiquidacionBatchResult, LiquidacionDetalle, LiquidacionFiltroDto,
    LiquidacionInput, LiquidacionListItem, LiquidacionSugerencia, LiquidacionSugerenciaQuery,
    LiquidacionUpdateInput,
};
use certaro_application::PagedResult;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn liquidaciones_list(
    state: State<'_, AppState>,
    query: ListQuery<LiquidacionFiltroDto>,
) -> ApiResult<PagedResult<LiquidacionListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.liquidaciones.list(query).await,
        Err(e) => Err(e),
    };
    handle("liquidaciones_list", outcome)
}

#[tauri::command]
pub async fn liquidaciones_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> ApiResult<LiquidacionDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.liquidaciones.get(id).await,
        Err(e) => Err(e),
    };
    handle("liquidaciones_get", outcome)
}

#[tauri::command]
pub async fn liquidaciones_suggest(
    state: State<'_, AppState>,
    query: LiquidacionSugerenciaQuery,
) -> ApiResult<Vec<LiquidacionSugerencia>> {
    let outcome = match state.services() {
        Ok(services) => services.liquidaciones.suggest(query).await,
        Err(e) => Err(e),
    };
    handle("liquidaciones_suggest", outcome)
}

#[tauri::command]
pub async fn liquidaciones_create(
    state: State<'_, AppState>,
    dto: LiquidacionInput,
) -> ApiResult<LiquidacionDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.liquidaciones.create(dto).await,
        Err(e) => Err(e),
    };
    handle("liquidaciones_create", outcome)
}

/// Atomic: one transaction for every settlement and every advance line. If one fails, none is
/// saved and the error says which with `params.empleado`.
#[tauri::command]
pub async fn liquidaciones_create_batch(
    state: State<'_, AppState>,
    dto: LiquidacionBatchInput,
) -> ApiResult<LiquidacionBatchResult> {
    let outcome = match state.services() {
        Ok(services) => services.liquidaciones.create_batch(dto).await,
        Err(e) => Err(e),
    };
    handle("liquidaciones_create_batch", outcome)
}

#[tauri::command]
pub async fn liquidaciones_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: LiquidacionUpdateInput,
    row_version: String,
) -> ApiResult<LiquidacionDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.liquidaciones.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("liquidaciones_update", outcome)
}

#[tauri::command]
pub async fn liquidaciones_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.liquidaciones.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("liquidaciones_delete", outcome)
}
