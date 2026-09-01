//! Commands of `adjuntos`. See `docs/11-contratos-tauri.md` §5.12.

use certaro_application::dtos::adjuntos::{AdjuntoInput, AdjuntoItem};
use certaro_domain::entities::EntidadAdjunto;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn adjuntos_list(
    state: State<'_, AppState>,
    entidad_tipo: EntidadAdjunto,
    entidad_id: Uuid,
) -> ApiResult<Vec<AdjuntoItem>> {
    let outcome = match state.services() {
        Ok(services) => services.adjuntos.list(entidad_tipo, entidad_id).await,
        Err(e) => Err(e),
    };
    handle("adjuntos_list", outcome)
}

#[tauri::command]
pub async fn adjuntos_add(
    state: State<'_, AppState>,
    input: AdjuntoInput,
) -> ApiResult<AdjuntoItem> {
    let outcome = match state.services() {
        Ok(services) => services.adjuntos.add(input).await,
        Err(e) => Err(e),
    };
    handle("adjuntos_add", outcome)
}

#[tauri::command]
pub async fn adjuntos_delete(state: State<'_, AppState>, id: Uuid) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.adjuntos.delete(id).await,
        Err(e) => Err(e),
    };
    handle("adjuntos_delete", outcome)
}

#[tauri::command]
pub async fn adjuntos_open(state: State<'_, AppState>, id: Uuid) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.adjuntos.open(id).await,
        Err(e) => Err(e),
    };
    handle("adjuntos_open", outcome)
}

#[tauri::command]
pub async fn adjuntos_reveal(state: State<'_, AppState>, id: Uuid) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.adjuntos.reveal(id).await,
        Err(e) => Err(e),
    };
    handle("adjuntos_reveal", outcome)
}
