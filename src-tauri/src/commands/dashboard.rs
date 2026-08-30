//! Commands of the `dashboard`. See `docs/11-contratos-tauri.md` §5.10.
//!
//! The alerts are a separate command from the statistics: they refresh on their own schedule, and
//! folding them into `dashboard_stats` would recompute every ranking to update a badge.

use eo_application::dtos::cotizaciones::Cotizacion;
use eo_application::dtos::dashboard::{Alerta, DashboardStats, PeriodoDashboard};
use tauri::State;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn dashboard_stats(
    state: State<'_, AppState>,
    periodo: PeriodoDashboard,
) -> ApiResult<DashboardStats> {
    let outcome = match state.services() {
        Ok(services) => services.dashboard.stats(periodo).await,
        Err(e) => Err(e),
    };
    handle("dashboard_stats", outcome)
}

#[tauri::command]
pub async fn dashboard_alertas(
    state: State<'_, AppState>,
    periodo: PeriodoDashboard,
) -> ApiResult<Vec<Alerta>> {
    let outcome = match state.services() {
        Ok(services) => services.dashboard.alertas(periodo).await,
        Err(e) => Err(e),
    };
    handle("dashboard_alertas", outcome)
}

/// The visible houses, from the cache while it is fresh. Never an error the screen has to show: an
/// unreachable service yields an empty list and the block simply does not appear.
#[tauri::command]
pub async fn cotizaciones_get(state: State<'_, AppState>) -> ApiResult<Vec<Cotizacion>> {
    let outcome = match state.services() {
        Ok(services) => services.cotizaciones.list().await,
        Err(e) => Err(e),
    };
    handle("cotizaciones_get", outcome)
}

#[tauri::command]
pub async fn cotizaciones_refresh(state: State<'_, AppState>) -> ApiResult<Vec<Cotizacion>> {
    let outcome = match state.services() {
        Ok(services) => services.cotizaciones.refresh().await,
        Err(e) => Err(e),
    };
    handle("cotizaciones_refresh", outcome)
}
