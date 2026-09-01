//! Commands of the commercial analysis. See `docs/11-contratos-tauri.md` §5.2 y §5.3.

use certaro_application::dtos::comercial::{
    AntiguedadDeuda, AntiguedadDeudaQuery, CuentaCorriente, CuentaCorrienteQuery,
};
use certaro_application::dtos::dashboard::RentabilidadItem;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn clientes_cuenta_corriente(
    state: State<'_, AppState>,
    query: CuentaCorrienteQuery,
) -> ApiResult<CuentaCorriente> {
    let outcome = match state.services() {
        Ok(services) => services.comercial.cuenta_corriente(query).await,
        Err(e) => Err(e),
    };
    handle("clientes_cuenta_corriente", outcome)
}

#[tauri::command]
pub async fn clientes_antiguedad_deuda(
    state: State<'_, AppState>,
    query: AntiguedadDeudaQuery,
) -> ApiResult<AntiguedadDeuda> {
    let outcome = match state.services() {
        Ok(services) => services.comercial.antiguedad_deuda(query).await,
        Err(e) => Err(e),
    };
    handle("clientes_antiguedad_deuda", outcome)
}

#[tauri::command]
pub async fn obras_rentabilidad(
    state: State<'_, AppState>,
    limite: Option<u64>,
) -> ApiResult<Vec<RentabilidadItem>> {
    let outcome = match state.services() {
        Ok(services) => services.comercial.rentabilidad_obras(limite).await,
        Err(e) => Err(e),
    };
    handle("obras_rentabilidad", outcome)
}

#[tauri::command]
pub async fn trabajos_rentabilidad(
    state: State<'_, AppState>,
    obra_id: Option<Uuid>,
    limite: Option<u64>,
) -> ApiResult<Vec<RentabilidadItem>> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .comercial
                .rentabilidad_trabajos(obra_id, limite)
                .await
        }
        Err(e) => Err(e),
    };
    handle("trabajos_rentabilidad", outcome)
}
