//! Commands of `obras`. See `docs/11-contratos-tauri.md` §5.5.

use certaro_application::dtos::common::{ListQuery, LookupItem};
use certaro_application::dtos::obras::{ObraDetalle, ObraFiltroDto, ObraInput, ObraListItem};
use certaro_application::PagedResult;
use certaro_domain::EstadoObra;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn obras_list(
    state: State<'_, AppState>,
    query: ListQuery<ObraFiltroDto>,
) -> ApiResult<PagedResult<ObraListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.obras.list(query).await,
        Err(e) => Err(e),
    };
    handle("obras_list", outcome)
}

#[tauri::command]
pub async fn obras_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<ObraDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.obras.get(id).await,
        Err(e) => Err(e),
    };
    handle("obras_get", outcome)
}

#[tauri::command]
pub async fn obras_create(state: State<'_, AppState>, dto: ObraInput) -> ApiResult<ObraDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.obras.create(dto).await,
        Err(e) => Err(e),
    };
    handle("obras_create", outcome)
}

#[tauri::command]
pub async fn obras_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: ObraInput,
    row_version: String,
) -> ApiResult<ObraDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.obras.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("obras_update", outcome)
}

/// Moves the site along its state machine. `cascada` is what the confirmation dialog answers: it
/// closes the open jobs along with the site instead of refusing the change.
#[tauri::command]
pub async fn obras_transition(
    state: State<'_, AppState>,
    id: Uuid,
    destino: EstadoObra,
    row_version: String,
    cascada: Option<bool>,
) -> ApiResult<ObraDetalle> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .obras
                .transition(id, destino, &row_version, cascada.unwrap_or(false))
                .await
        }
        Err(e) => Err(e),
    };
    handle("obras_transition", outcome)
}

#[tauri::command]
pub async fn obras_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.obras.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("obras_delete", outcome)
}

#[tauri::command]
pub async fn obras_lookup(
    state: State<'_, AppState>,
    cliente_id: Option<Uuid>,
    texto: Option<String>,
    limite: Option<u64>,
) -> ApiResult<Vec<LookupItem>> {
    let outcome = match state.services() {
        Ok(services) => services.obras.lookup(cliente_id, texto, limite).await,
        Err(e) => Err(e),
    };
    handle("obras_lookup", outcome)
}

/// The number the create form pre-fills with, so the user never has to guess it.
#[tauri::command]
pub async fn obras_siguiente_numero(state: State<'_, AppState>) -> ApiResult<i32> {
    let outcome = match state.services() {
        Ok(services) => services.obras.siguiente_numero().await,
        Err(e) => Err(e),
    };
    handle("obras_siguiente_numero", outcome)
}
