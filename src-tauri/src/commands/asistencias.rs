//! Commands of `asistencia`. See `docs/11-contratos-tauri.md` §5.7.
//!
//! `asistencia_upsert` takes no `rowVersion`: `(empleadoId, fecha)` is the identity and the last
//! click wins. It is a fast entry grid, not a form with concurrency.

use chrono::NaiveDate;
use certaro_application::dtos::asistencias::{
    AsistenciaCelda, AsistenciaGrilla, AsistenciaGrillaQuery, AsistenciaRangoInput,
    AsistenciaUpsertInput,
};
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn asistencia_grilla(
    state: State<'_, AppState>,
    query: AsistenciaGrillaQuery,
) -> ApiResult<AsistenciaGrilla> {
    let outcome = match state.services() {
        Ok(services) => services.asistencias.grilla(query).await,
        Err(e) => Err(e),
    };
    handle("asistencia_grilla", outcome)
}

#[tauri::command]
pub async fn asistencia_upsert(
    state: State<'_, AppState>,
    dto: AsistenciaUpsertInput,
) -> ApiResult<AsistenciaCelda> {
    let outcome = match state.services() {
        Ok(services) => services.asistencias.upsert(dto).await,
        Err(e) => Err(e),
    };
    handle("asistencia_upsert", outcome)
}

#[tauri::command]
pub async fn asistencia_upsert_rango(
    state: State<'_, AppState>,
    dto: AsistenciaRangoInput,
) -> ApiResult<Vec<AsistenciaCelda>> {
    let outcome = match state.services() {
        Ok(services) => services.asistencias.upsert_rango(dto).await,
        Err(e) => Err(e),
    };
    handle("asistencia_upsert_rango", outcome)
}

#[tauri::command]
pub async fn asistencia_delete(
    state: State<'_, AppState>,
    empleado_id: Uuid,
    fecha: NaiveDate,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.asistencias.delete(empleado_id, fecha).await,
        Err(e) => Err(e),
    };
    handle("asistencia_delete", outcome)
}
