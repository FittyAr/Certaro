//! Commands of `certificados`. See `docs/11-contratos-tauri.md` §5.5.
//!
//! There is no full `update`: an issued certificate is immutable except for its notes, so the only
//! write besides issuing and voiding is `certificados_update_observaciones`.

use certaro_application::dtos::certificados::{
    CertificadoBorrador, CertificadoDetalle, CertificadoFiltroDto, CertificadoInput,
    CertificadoListItem,
};
use certaro_application::dtos::common::ListQuery;
use certaro_application::PagedResult;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn certificados_list(
    state: State<'_, AppState>,
    query: ListQuery<CertificadoFiltroDto>,
) -> ApiResult<PagedResult<CertificadoListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.certificados.list(query).await,
        Err(e) => Err(e),
    };
    handle("certificados_list", outcome)
}

#[tauri::command]
pub async fn certificados_get(
    state: State<'_, AppState>,
    id: Uuid,
) -> ApiResult<CertificadoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.certificados.get(id).await,
        Err(e) => Err(e),
    };
    handle("certificados_get", outcome)
}

/// Prefills the issuing form with each line's certified history and what is left of it.
#[tauri::command]
pub async fn certificados_preparar(
    state: State<'_, AppState>,
    orden_trabajo_id: Uuid,
) -> ApiResult<CertificadoBorrador> {
    let outcome = match state.services() {
        Ok(services) => services.certificados.preparar(orden_trabajo_id).await,
        Err(e) => Err(e),
    };
    handle("certificados_preparar", outcome)
}

#[tauri::command]
pub async fn certificados_create(
    state: State<'_, AppState>,
    dto: CertificadoInput,
) -> ApiResult<CertificadoDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.certificados.create(dto).await,
        Err(e) => Err(e),
    };
    handle("certificados_create", outcome)
}

#[tauri::command]
pub async fn certificados_update_observaciones(
    state: State<'_, AppState>,
    id: Uuid,
    observaciones: Option<String>,
    row_version: String,
) -> ApiResult<CertificadoDetalle> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .certificados
                .update_observaciones(id, observaciones, &row_version)
                .await
        }
        Err(e) => Err(e),
    };
    handle("certificados_update_observaciones", outcome)
}

#[tauri::command]
pub async fn certificados_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.certificados.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("certificados_delete", outcome)
}
