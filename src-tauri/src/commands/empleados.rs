//! Commands of `empleados`. See `docs/11-contratos-tauri.md` §5.7.

use eo_application::dtos::common::{ListQuery, LookupItem};
use eo_application::dtos::empleados::{
    EmpleadoDetalle, EmpleadoFiltroDto, EmpleadoInput, EmpleadoListItem,
};
use eo_application::PagedResult;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn empleados_list(
    state: State<'_, AppState>,
    query: ListQuery<EmpleadoFiltroDto>,
) -> ApiResult<PagedResult<EmpleadoListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.empleados.list(query).await,
        Err(e) => Err(e),
    };
    handle("empleados_list", outcome)
}

#[tauri::command]
pub async fn empleados_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<EmpleadoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.empleados.get(id).await,
        Err(e) => Err(e),
    };
    handle("empleados_get", outcome)
}

#[tauri::command]
pub async fn empleados_lookup(
    state: State<'_, AppState>,
    solo_activos: Option<bool>,
    texto: Option<String>,
    limite: Option<u64>,
) -> ApiResult<Vec<LookupItem>> {
    let outcome = match state.services() {
        Ok(services) => services.empleados.lookup(solo_activos, texto, limite).await,
        Err(e) => Err(e),
    };
    handle("empleados_lookup", outcome)
}

/// The distinct roles already in use, so the filter offers what exists instead of a free field.
#[tauri::command]
pub async fn empleados_cargos(state: State<'_, AppState>) -> ApiResult<Vec<String>> {
    let outcome = match state.services() {
        Ok(services) => services.empleados.cargos().await,
        Err(e) => Err(e),
    };
    handle("empleados_cargos", outcome)
}

#[tauri::command]
pub async fn empleados_create(
    state: State<'_, AppState>,
    dto: EmpleadoInput,
) -> ApiResult<EmpleadoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.empleados.create(dto).await,
        Err(e) => Err(e),
    };
    handle("empleados_create", outcome)
}

#[tauri::command]
pub async fn empleados_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: EmpleadoInput,
    row_version: String,
) -> ApiResult<EmpleadoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.empleados.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("empleados_update", outcome)
}

#[tauri::command]
pub async fn empleados_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.empleados.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("empleados_delete", outcome)
}
