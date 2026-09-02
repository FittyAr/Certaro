//! Tauri commands for the Calendar and Scheduler module.

use certaro_application::dtos::calendario::{
    ActualizarEventoInput, ActualizarGrupoRecursoInput, ActualizarRecursoInput,
    CalendarioEventoDto, CalendarioGrupoRecursoDto, CalendarioRecursoDto, CrearEventoInput,
    CrearGrupoRecursoInput, CrearRecursoInput,
};
use certaro_domain::RowVersion;
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

// =========================================================================
// Grupos de Recurso
// =========================================================================

#[tauri::command]
pub async fn calendario_list_grupos(
    state: State<'_, AppState>,
) -> ApiResult<Vec<CalendarioGrupoRecursoDto>> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.list_grupos().await,
        Err(e) => Err(e),
    };
    handle("calendario_list_grupos", outcome)
}

#[tauri::command]
pub async fn calendario_create_grupo(
    state: State<'_, AppState>,
    input: CrearGrupoRecursoInput,
) -> ApiResult<CalendarioGrupoRecursoDto> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.create_grupo(input).await,
        Err(e) => Err(e),
    };
    handle("calendario_create_grupo", outcome)
}

#[tauri::command]
pub async fn calendario_update_grupo(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarGrupoRecursoInput,
) -> ApiResult<CalendarioGrupoRecursoDto> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.update_grupo(id, input).await,
        Err(e) => Err(e),
    };
    handle("calendario_update_grupo", outcome)
}

#[tauri::command]
pub async fn calendario_delete_grupo(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match (state.services(), RowVersion::parse_hex(&row_version)) {
        (Ok(s), Ok(rv)) => s.calendario.delete_grupo(id, rv).await,
        (Ok(_), Err(_)) => Err(certaro_application::AppError::Concurrency {
            entity: "calendario_grupos_recurso",
        }),
        (Err(e), _) => Err(e),
    };
    handle("calendario_delete_grupo", outcome)
}

// =========================================================================
// Recursos
// =========================================================================

#[tauri::command]
pub async fn calendario_list_recursos(
    state: State<'_, AppState>,
) -> ApiResult<Vec<CalendarioRecursoDto>> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.list_recursos().await,
        Err(e) => Err(e),
    };
    handle("calendario_list_recursos", outcome)
}

#[tauri::command]
pub async fn calendario_create_recurso(
    state: State<'_, AppState>,
    input: CrearRecursoInput,
) -> ApiResult<CalendarioRecursoDto> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.create_recurso(input).await,
        Err(e) => Err(e),
    };
    handle("calendario_create_recurso", outcome)
}

#[tauri::command]
pub async fn calendario_update_recurso(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarRecursoInput,
) -> ApiResult<CalendarioRecursoDto> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.update_recurso(id, input).await,
        Err(e) => Err(e),
    };
    handle("calendario_update_recurso", outcome)
}

#[tauri::command]
pub async fn calendario_delete_recurso(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match (state.services(), RowVersion::parse_hex(&row_version)) {
        (Ok(s), Ok(rv)) => s.calendario.delete_recurso(id, rv).await,
        (Ok(_), Err(_)) => Err(certaro_application::AppError::Concurrency {
            entity: "calendario_recursos",
        }),
        (Err(e), _) => Err(e),
    };
    handle("calendario_delete_recurso", outcome)
}

#[tauri::command]
pub async fn calendario_sincronizar_empleados(state: State<'_, AppState>) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.sincronizar_empleados_a_recursos().await,
        Err(e) => Err(e),
    };
    handle("calendario_sincronizar_empleados", outcome)
}

// =========================================================================
// Eventos
// =========================================================================

#[tauri::command]
pub async fn calendario_list_eventos(
    state: State<'_, AppState>,
    desde: String,
    hasta: String,
) -> ApiResult<Vec<CalendarioEventoDto>> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.list_eventos(&desde, &hasta).await,
        Err(e) => Err(e),
    };
    handle("calendario_list_eventos", outcome)
}

#[tauri::command]
pub async fn calendario_create_evento(
    state: State<'_, AppState>,
    input: CrearEventoInput,
) -> ApiResult<CalendarioEventoDto> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.create_evento(input).await,
        Err(e) => Err(e),
    };
    handle("calendario_create_evento", outcome)
}

#[tauri::command]
pub async fn calendario_update_evento(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarEventoInput,
) -> ApiResult<CalendarioEventoDto> {
    let outcome = match state.services() {
        Ok(s) => s.calendario.update_evento(id, input).await,
        Err(e) => Err(e),
    };
    handle("calendario_update_evento", outcome)
}

#[tauri::command]
pub async fn calendario_mover_evento(
    state: State<'_, AppState>,
    id: Uuid,
    nuevo_inicio: String,
    nuevo_fin: String,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match (state.services(), RowVersion::parse_hex(&row_version)) {
        (Ok(s), Ok(rv)) => {
            s.calendario
                .mover_evento(id, &nuevo_inicio, &nuevo_fin, rv)
                .await
        }
        (Ok(_), Err(_)) => Err(certaro_application::AppError::Concurrency {
            entity: "calendario_eventos",
        }),
        (Err(e), _) => Err(e),
    };
    handle("calendario_mover_evento", outcome)
}

#[tauri::command]
pub async fn calendario_delete_evento(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match (state.services(), RowVersion::parse_hex(&row_version)) {
        (Ok(s), Ok(rv)) => s.calendario.delete_evento(id, rv).await,
        (Ok(_), Err(_)) => Err(certaro_application::AppError::Concurrency {
            entity: "calendario_eventos",
        }),
        (Err(e), _) => Err(e),
    };
    handle("calendario_delete_evento", outcome)
}
