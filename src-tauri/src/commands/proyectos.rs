//! Commands of `proyectos`. See `docs/11-contratos-tauri.md` §5.5.

use certaro_application::dtos::common::{ListQuery, LookupItem};
use certaro_application::dtos::proyectos::{ProyectoDetalle, ProyectoFiltroDto, ProyectoInput, ProyectoListItem};
use certaro_application::PagedResult;
use certaro_domain::EstadoProyecto;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn proyectos_list(
    state: State<'_, AppState>,
    query: ListQuery<ProyectoFiltroDto>,
) -> ApiResult<PagedResult<ProyectoListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.proyectos.list(query).await,
        Err(e) => Err(e),
    };
    handle("proyectos_list", outcome)
}

#[tauri::command]
pub async fn proyectos_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<ProyectoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.proyectos.get(id).await,
        Err(e) => Err(e),
    };
    handle("proyectos_get", outcome)
}

#[tauri::command]
pub async fn proyectos_create(state: State<'_, AppState>, dto: ProyectoInput) -> ApiResult<ProyectoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.proyectos.create(dto).await,
        Err(e) => Err(e),
    };
    handle("proyectos_create", outcome)
}

#[tauri::command]
pub async fn proyectos_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: ProyectoInput,
    row_version: String,
) -> ApiResult<ProyectoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.proyectos.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("proyectos_update", outcome)
}

/// Moves the site along its state machine. `cascada` is what the confirmation dialog answers: it
/// closes the open jobs along with the site instead of refusing the change.
#[tauri::command]
pub async fn proyectos_transition(
    state: State<'_, AppState>,
    id: Uuid,
    destino: EstadoProyecto,
    row_version: String,
    cascada: Option<bool>,
) -> ApiResult<ProyectoDetalle> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .proyectos
                .transition(id, destino, &row_version, cascada.unwrap_or(false))
                .await
        }
        Err(e) => Err(e),
    };
    handle("proyectos_transition", outcome)
}

#[tauri::command]
pub async fn proyectos_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.proyectos.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("proyectos_delete", outcome)
}

#[tauri::command]
pub async fn proyectos_lookup(
    state: State<'_, AppState>,
    cliente_id: Option<Uuid>,
    texto: Option<String>,
    limite: Option<u64>,
) -> ApiResult<Vec<LookupItem>> {
    let outcome = match state.services() {
        Ok(services) => services.proyectos.lookup(cliente_id, texto, limite).await,
        Err(e) => Err(e),
    };
    handle("proyectos_lookup", outcome)
}

/// The number the create form pre-fills with, so the user never has to guess it.
#[tauri::command]
pub async fn proyectos_siguiente_numero(state: State<'_, AppState>) -> ApiResult<i32> {
    let outcome = match state.services() {
        Ok(services) => services.proyectos.siguiente_numero().await,
        Err(e) => Err(e),
    };
    handle("proyectos_siguiente_numero", outcome)
}
