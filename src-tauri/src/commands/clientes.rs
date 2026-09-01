//! Commands of `clientes`. See `docs/11-contratos-tauri.md` §5.4.

use certaro_application::dtos::clientes::{
    ClienteDetalle, ClienteFiltroDto, ClienteInput, ClienteListItem,
};
use certaro_application::dtos::common::{ListQuery, LookupItem};
use certaro_application::PagedResult;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn clientes_list(
    state: State<'_, AppState>,
    query: ListQuery<ClienteFiltroDto>,
) -> ApiResult<PagedResult<ClienteListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.clientes.list(query).await,
        Err(e) => Err(e),
    };
    handle("clientes_list", outcome)
}

#[tauri::command]
pub async fn clientes_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<ClienteDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.clientes.get(id).await,
        Err(e) => Err(e),
    };
    handle("clientes_get", outcome)
}

#[tauri::command]
pub async fn clientes_create(
    state: State<'_, AppState>,
    dto: ClienteInput,
) -> ApiResult<ClienteDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.clientes.create(dto).await,
        Err(e) => Err(e),
    };
    handle("clientes_create", outcome)
}

#[tauri::command]
pub async fn clientes_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: ClienteInput,
    row_version: String,
) -> ApiResult<ClienteDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.clientes.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("clientes_update", outcome)
}

#[tauri::command]
pub async fn clientes_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.clientes.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("clientes_delete", outcome)
}

#[tauri::command]
pub async fn clientes_lookup(
    state: State<'_, AppState>,
    texto: Option<String>,
    limite: Option<u64>,
) -> ApiResult<Vec<LookupItem>> {
    let outcome = match state.services() {
        Ok(services) => services.clientes.lookup(texto, limite).await,
        Err(e) => Err(e),
    };
    handle("clientes_lookup", outcome)
}
