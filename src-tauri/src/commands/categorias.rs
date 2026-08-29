//! Commands of `categorias`. See `docs/11-contratos-tauri.md` §5.12.

use eo_application::dtos::categorias::{
    CategoriaDetalle, CategoriaFiltroDto, CategoriaInput, CategoriaListItem,
};
use eo_application::dtos::common::{ListQuery, LookupItem};
use eo_application::PagedResult;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn categorias_list(
    state: State<'_, AppState>,
    query: ListQuery<CategoriaFiltroDto>,
) -> ApiResult<PagedResult<CategoriaListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.categorias.list(query).await,
        Err(e) => Err(e),
    };
    handle("categorias_list", outcome)
}

#[tauri::command]
pub async fn categorias_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<CategoriaDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.categorias.get(id).await,
        Err(e) => Err(e),
    };
    handle("categorias_get", outcome)
}

#[tauri::command]
pub async fn categorias_create(
    state: State<'_, AppState>,
    dto: CategoriaInput,
) -> ApiResult<CategoriaDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.categorias.create(dto).await,
        Err(e) => Err(e),
    };
    handle("categorias_create", outcome)
}

#[tauri::command]
pub async fn categorias_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: CategoriaInput,
    row_version: String,
) -> ApiResult<CategoriaDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.categorias.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("categorias_update", outcome)
}

#[tauri::command]
pub async fn categorias_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.categorias.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("categorias_delete", outcome)
}

#[tauri::command]
pub async fn categorias_lookup(
    state: State<'_, AppState>,
    texto: Option<String>,
    limite: Option<u64>,
) -> ApiResult<Vec<LookupItem>> {
    let outcome = match state.services() {
        Ok(services) => services.categorias.lookup(texto, limite).await,
        Err(e) => Err(e),
    };
    handle("categorias_lookup", outcome)
}
