//! Commands of `trabajos`. See `docs/11-contratos-tauri.md` §5.6.

use certaro_application::dtos::common::{ListQuery, LookupItem};
use certaro_application::dtos::trabajos::{
    TrabajoDetalle, TrabajoFiltroDto, TrabajoInput, TrabajoListItem,
};
use certaro_application::PagedResult;
use certaro_domain::EstadoTrabajo;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn trabajos_list(
    state: State<'_, AppState>,
    query: ListQuery<TrabajoFiltroDto>,
) -> ApiResult<PagedResult<TrabajoListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.trabajos.list(query).await,
        Err(e) => Err(e),
    };
    handle("trabajos_list", outcome)
}

#[tauri::command]
pub async fn trabajos_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<TrabajoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.trabajos.get(id).await,
        Err(e) => Err(e),
    };
    handle("trabajos_get", outcome)
}

#[tauri::command]
pub async fn trabajos_create(
    state: State<'_, AppState>,
    dto: TrabajoInput,
) -> ApiResult<TrabajoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.trabajos.create(dto).await,
        Err(e) => Err(e),
    };
    handle("trabajos_create", outcome)
}

#[tauri::command]
pub async fn trabajos_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: TrabajoInput,
    row_version: String,
) -> ApiResult<TrabajoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.trabajos.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("trabajos_update", outcome)
}

#[tauri::command]
pub async fn trabajos_transition(
    state: State<'_, AppState>,
    id: Uuid,
    destino: EstadoTrabajo,
    row_version: String,
    forzar: Option<bool>,
) -> ApiResult<TrabajoDetalle> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .trabajos
                .transition(id, destino, &row_version, forzar.unwrap_or(false))
                .await
        }
        Err(e) => Err(e),
    };
    handle("trabajos_transition", outcome)
}

#[tauri::command]
pub async fn trabajos_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.trabajos.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("trabajos_delete", outcome)
}

#[tauri::command]
pub async fn trabajos_lookup(
    state: State<'_, AppState>,
    proyecto_id: Option<Uuid>,
    texto: Option<String>,
    limite: Option<u64>,
) -> ApiResult<Vec<LookupItem>> {
    let outcome = match state.services() {
        Ok(services) => services.trabajos.lookup(proyecto_id, texto, limite).await,
        Err(e) => Err(e),
    };
    handle("trabajos_lookup", outcome)
}
