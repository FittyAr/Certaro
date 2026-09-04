//! Commands of `ordenes_trabajo`. See `docs/11-contratos-tauri.md` §5.4.
//!
//! The list is not paged: a job has a handful of sheets and each sheet tens of lines.

use certaro_application::dtos::common::LookupItem;
use certaro_application::dtos::ordenes_trabajo::{
    OrdenTrabajoDetalle, OrdenTrabajoInput, OrdenTrabajoListItem,
};
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn ordenes_trabajo_list(
    state: State<'_, AppState>,
    trabajo_id: Option<Uuid>,
) -> ApiResult<Vec<OrdenTrabajoListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.ordenes_trabajo.listar(trabajo_id).await,
        Err(e) => Err(e),
    };
    handle("ordenes_trabajo_list", outcome)
}

#[tauri::command]
pub async fn ordenes_trabajo_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> ApiResult<OrdenTrabajoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.ordenes_trabajo.get(id).await,
        Err(e) => Err(e),
    };
    handle("ordenes_trabajo_get", outcome)
}

#[tauri::command]
pub async fn ordenes_trabajo_create(
    state: State<'_, AppState>,
    dto: OrdenTrabajoInput,
) -> ApiResult<OrdenTrabajoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.ordenes_trabajo.create(dto).await,
        Err(e) => Err(e),
    };
    handle("ordenes_trabajo_create", outcome)
}

#[tauri::command]
pub async fn ordenes_trabajo_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: OrdenTrabajoInput,
    row_version: String,
) -> ApiResult<OrdenTrabajoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.ordenes_trabajo.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("ordenes_trabajo_update", outcome)
}

#[tauri::command]
pub async fn ordenes_trabajo_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.ordenes_trabajo.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("ordenes_trabajo_delete", outcome)
}

#[tauri::command]
pub async fn ordenes_trabajo_lookup(
    state: State<'_, AppState>,
    trabajo_id: Option<Uuid>,
    texto: Option<String>,
    limite: Option<u64>,
) -> ApiResult<Vec<LookupItem>> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .ordenes_trabajo
                .lookup(trabajo_id, texto, limite)
                .await
        }
        Err(e) => Err(e),
    };
    handle("ordenes_trabajo_lookup", outcome)
}
