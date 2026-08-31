//! Commands of `reportes`. See `docs/11-contratos-tauri.md` §5.11.
//!
//! One command per report family, all of them taking the destination path the frontend got from
//! the system dialog. There is no command that decides a path on its own.

use eo_application::dtos::movimientos::MovimientoFiltroDto;
use eo_application::dtos::reportes::{ExportResult, FormatoExport, ReporteRequest};
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn movimientos_export(
    state: State<'_, AppState>,
    filtro: MovimientoFiltroDto,
    formato: FormatoExport,
    destino: String,
) -> ApiResult<ExportResult> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .reportes
                .generar(ReporteRequest::Movimientos { filtro }, formato, destino)
                .await
        }
        Err(e) => Err(e),
    };
    handle("movimientos_export", outcome)
}

#[tauri::command]
pub async fn liquidacion_export(
    state: State<'_, AppState>,
    id: Uuid,
    destino: String,
) -> ApiResult<ExportResult> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .reportes
                .generar(
                    ReporteRequest::Liquidacion { id },
                    FormatoExport::Pdf,
                    destino,
                )
                .await
        }
        Err(e) => Err(e),
    };
    handle("liquidacion_export", outcome)
}

#[tauri::command]
pub async fn certificado_export(
    state: State<'_, AppState>,
    id: Uuid,
    destino: String,
) -> ApiResult<ExportResult> {
    let outcome = match state.services() {
        Ok(services) => {
            services
                .reportes
                .generar(
                    ReporteRequest::Certificado { id },
                    FormatoExport::Pdf,
                    destino,
                )
                .await
        }
        Err(e) => Err(e),
    };
    handle("certificado_export", outcome)
}

/// The suggested filename, so the dialog opens already filled in and the user does not have to
/// invent a name that later nobody can find.
#[tauri::command]
pub async fn reportes_nombre_sugerido(
    state: State<'_, AppState>,
    reporte: String,
    formato: FormatoExport,
    detalle: Option<String>,
) -> ApiResult<String> {
    let outcome = match state.services() {
        Ok(services) => services
            .reportes
            .nombre_sugerido(&reporte, formato, detalle.as_deref()),
        Err(e) => Err(e),
    };
    handle("reportes_nombre_sugerido", outcome)
}
