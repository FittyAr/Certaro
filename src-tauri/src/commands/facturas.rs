//! Commands of `facturas` and their payments. See `docs/11-contratos-tauri.md` §5.8.
//!
//! Every payment command answers with the whole invoice: a payment changes the balance and can
//! change the state, and returning only the payment would leave the screen showing stale totals.

use certaro_application::dtos::common::{ListQuery, LookupItem};
use certaro_application::dtos::facturas::{
    FacturaDetalle, FacturaFiltroDto, FacturaInput, FacturaListItem, PagoFacturaInput,
    PagoFacturaItem,
};
use certaro_application::PagedResult;
use certaro_domain::EstadoFactura;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn facturas_list(
    state: State<'_, AppState>,
    query: ListQuery<FacturaFiltroDto>,
) -> ApiResult<PagedResult<FacturaListItem>> {
    let outcome = match state.services() {
        Ok(services) => services.facturas.list(query).await,
        Err(e) => Err(e),
    };
    handle("facturas_list", outcome)
}

#[tauri::command]
pub async fn facturas_get(state: State<'_, AppState>, id: Uuid) -> ApiResult<FacturaDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.facturas.get(id).await,
        Err(e) => Err(e),
    };
    handle("facturas_get", outcome)
}

#[tauri::command]
pub async fn facturas_create(
    state: State<'_, AppState>,
    dto: FacturaInput,
) -> ApiResult<FacturaDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.facturas.create(dto).await,
        Err(e) => Err(e),
    };
    handle("facturas_create", outcome)
}

#[tauri::command]
pub async fn facturas_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: FacturaInput,
    row_version: String,
) -> ApiResult<FacturaDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.facturas.update(id, dto, &row_version).await,
        Err(e) => Err(e),
    };
    handle("facturas_update", outcome)
}

#[tauri::command]
pub async fn facturas_transition(
    state: State<'_, AppState>,
    id: Uuid,
    destino: EstadoFactura,
    row_version: String,
) -> ApiResult<FacturaDetalle> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .facturas
                .transition(id, destino, &row_version)
                .await
        }
        Err(e) => Err(e),
    };
    handle("facturas_transition", outcome)
}

#[tauri::command]
pub async fn facturas_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.facturas.delete(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("facturas_delete", outcome)
}

#[tauri::command]
pub async fn facturas_lookup(
    state: State<'_, AppState>,
    cliente_id: Option<Uuid>,
    solo_impagas: Option<bool>,
    texto: Option<String>,
    limite: Option<u64>,
) -> ApiResult<Vec<LookupItem>> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .facturas
                .lookup(cliente_id, solo_impagas.unwrap_or(false), texto, limite)
                .await
        }
        Err(e) => Err(e),
    };
    handle("facturas_lookup", outcome)
}

#[tauri::command]
pub async fn facturas_pagos(
    state: State<'_, AppState>,
    factura_id: Uuid,
) -> ApiResult<Vec<PagoFacturaItem>> {
    let outcome = match state.services() {
        Ok(services) => services.facturas.pagos_de(factura_id).await,
        Err(e) => Err(e),
    };
    handle("facturas_pagos", outcome)
}

#[tauri::command]
pub async fn facturas_pago_create(
    state: State<'_, AppState>,
    dto: PagoFacturaInput,
) -> ApiResult<FacturaDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.facturas.crear_pago(dto).await,
        Err(e) => Err(e),
    };
    handle("facturas_pago_create", outcome)
}

#[tauri::command]
pub async fn facturas_pago_update(
    state: State<'_, AppState>,
    id: Uuid,
    dto: PagoFacturaInput,
    row_version: String,
) -> ApiResult<FacturaDetalle> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .facturas
                .actualizar_pago(id, dto, &row_version)
                .await
        }
        Err(e) => Err(e),
    };
    handle("facturas_pago_update", outcome)
}

#[tauri::command]
pub async fn facturas_pago_delete(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<FacturaDetalle> {
    let outcome = match state.services() {
        Ok(services) => services.facturas.borrar_pago(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("facturas_pago_delete", outcome)
}
