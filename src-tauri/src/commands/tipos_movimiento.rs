//! Commands of `tipos_movimiento`. See `docs/11-contratos-tauri.md` §5.11.
//!
//! A command deserialises, delegates and wraps the error. It never validates, calculates or
//! touches SQL.

use eo_application::dtos::common::{ListQuery, LookupItem};
use eo_application::dtos::tipos_movimiento::{
    TipoMovimientoDetalle, TipoMovimientoFiltroDto, TipoMovimientoInput, TipoMovimientoListItem,
};
use eo_application::PagedResult;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn tipos_movimiento_list(
    state: State<'_, AppState>,
    query: ListQuery<TipoMovimientoFiltroDto>,
) -> ApiResult<PagedResult<TipoMovimientoListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.tipos_movimiento.list(query).await,
        Err(e) => Err(e),
    };
    handle("tipos_movimiento_list", outcome)
}

#[tauri::command]
pub async fn tipos_movimiento_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> ApiResult<TipoMovimientoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.tipos_movimiento.get(id).await,
        Err(e) => Err(e),
    };
    handle("tipos_movimiento_get", outcome)
}

#[tauri::command]
pub async fn tipos_movimiento_create(
    state: State<'_, AppState>,
    dto: TipoMovimientoInput,
) -> ApiResult<TipoMovimientoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.tipos_movimiento.create(dto).await,
        Err(e) => Err(e),
    };
    handle("tipos_movimiento_create", outcome)
}

#[tauri::command]
pub async fn tipos_movimiento_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: TipoMovimientoInput,
    row_version: String,
) -> ApiResult<TipoMovimientoDetalle> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .tipos_movimiento
                .update(id, dto, &row_version)
                .await
        }
        Err(e) => Err(e),
    };
    handle("tipos_movimiento_update", outcome)
}

#[tauri::command]
pub async fn tipos_movimiento_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.tipos_movimiento.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("tipos_movimiento_delete", outcome)
}

#[tauri::command]
pub async fn tipos_movimiento_lookup(
    state: State<'_, AppState>,
    texto: Option<String>,
    limite: Option<u64>,
) -> ApiResult<Vec<LookupItem>> {
    let outcome = match state.services() {
        Ok(services) => services.tipos_movimiento.lookup(texto, limite).await,
        Err(e) => Err(e),
    };
    handle("tipos_movimiento_lookup", outcome)
}
