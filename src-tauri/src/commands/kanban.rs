//! Tauri commands for the Kanban module.

use certaro_application::dtos::kanban::{
    ActualizarChecklistInput, ActualizarColumnaInput, ActualizarEtiquetaInput,
    ActualizarTableroInput, ActualizarTarjetaInput, CrearChecklistInput, CrearColumnaInput,
    CrearEtiquetaInput, CrearTableroInput, CrearTarjetaInput, KanbanChecklistDto, KanbanColumnaDto,
    KanbanEtiquetaDto, KanbanTableroDetalleDto, KanbanTableroDto, KanbanTarjetaDto,
    MoverTarjetaInput, ReordenarColumnasInput, ReordenarTarjetasInput,
};
use tauri::State;
use uuid::Uuid;

use crate::error::{handle, ApiResult};
use crate::state::AppState;

#[tauri::command]
pub async fn kanban_list_tableros(state: State<'_, AppState>) -> ApiResult<Vec<KanbanTableroDto>> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.list_tableros().await,
        Err(e) => Err(e),
    };
    handle("kanban_list_tableros", outcome)
}

#[tauri::command]
pub async fn kanban_get_tablero(
    state: State<'_, AppState>,
    id: Uuid,
) -> ApiResult<KanbanTableroDetalleDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.get_tablero_detalle(id).await,
        Err(e) => Err(e),
    };
    handle("kanban_get_tablero", outcome)
}

#[tauri::command]
pub async fn kanban_create_tablero(
    state: State<'_, AppState>,
    input: CrearTableroInput,
) -> ApiResult<KanbanTableroDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.create_tablero(input).await,
        Err(e) => Err(e),
    };
    handle("kanban_create_tablero", outcome)
}

#[tauri::command]
pub async fn kanban_update_tablero(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarTableroInput,
) -> ApiResult<KanbanTableroDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.update_tablero(id, input).await,
        Err(e) => Err(e),
    };
    handle("kanban_update_tablero", outcome)
}

#[tauri::command]
pub async fn kanban_delete_tablero(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.delete_tablero(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("kanban_delete_tablero", outcome)
}

#[tauri::command]
pub async fn kanban_create_columna(
    state: State<'_, AppState>,
    input: CrearColumnaInput,
) -> ApiResult<KanbanColumnaDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.create_columna(input).await,
        Err(e) => Err(e),
    };
    handle("kanban_create_columna", outcome)
}

#[tauri::command]
pub async fn kanban_update_columna(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarColumnaInput,
) -> ApiResult<KanbanColumnaDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.update_columna(id, input).await,
        Err(e) => Err(e),
    };
    handle("kanban_update_columna", outcome)
}

#[tauri::command]
pub async fn kanban_delete_columna(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.delete_columna(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("kanban_delete_columna", outcome)
}

#[tauri::command]
pub async fn kanban_create_tarjeta(
    state: State<'_, AppState>,
    input: CrearTarjetaInput,
) -> ApiResult<KanbanTarjetaDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.create_tarjeta(input).await,
        Err(e) => Err(e),
    };
    handle("kanban_create_tarjeta", outcome)
}

#[tauri::command]
pub async fn kanban_update_tarjeta(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarTarjetaInput,
) -> ApiResult<KanbanTarjetaDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.update_tarjeta(id, input).await,
        Err(e) => Err(e),
    };
    handle("kanban_update_tarjeta", outcome)
}

#[tauri::command]
pub async fn kanban_mover_tarjeta(
    state: State<'_, AppState>,
    input: MoverTarjetaInput,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.mover_tarjeta(input).await,
        Err(e) => Err(e),
    };
    handle("kanban_mover_tarjeta", outcome)
}

#[tauri::command]
pub async fn kanban_reordenar_columnas(
    state: State<'_, AppState>,
    input: ReordenarColumnasInput,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.reordenar_columnas(input).await,
        Err(e) => Err(e),
    };
    handle("kanban_reordenar_columnas", outcome)
}

#[tauri::command]
pub async fn kanban_reordenar_tarjetas(
    state: State<'_, AppState>,
    input: ReordenarTarjetasInput,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.reordenar_tarjetas(input).await,
        Err(e) => Err(e),
    };
    handle("kanban_reordenar_tarjetas", outcome)
}

#[tauri::command]
pub async fn kanban_delete_tarjeta(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.delete_tarjeta(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("kanban_delete_tarjeta", outcome)
}

#[tauri::command]
pub async fn kanban_sincronizar_preset(
    state: State<'_, AppState>,
    tablero_id: Uuid,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.sincronizar_preset(tablero_id).await,
        Err(e) => Err(e),
    };
    handle("kanban_sincronizar_preset", outcome)
}

#[tauri::command]
pub async fn kanban_list_etiquetas(
    state: State<'_, AppState>,
) -> ApiResult<Vec<KanbanEtiquetaDto>> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.list_etiquetas().await,
        Err(e) => Err(e),
    };
    handle("kanban_list_etiquetas", outcome)
}

#[tauri::command]
pub async fn kanban_create_etiqueta(
    state: State<'_, AppState>,
    input: CrearEtiquetaInput,
) -> ApiResult<KanbanEtiquetaDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.create_etiqueta(input).await,
        Err(e) => Err(e),
    };
    handle("kanban_create_etiqueta", outcome)
}

#[tauri::command]
pub async fn kanban_update_etiqueta(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarEtiquetaInput,
) -> ApiResult<KanbanEtiquetaDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.update_etiqueta(id, input).await,
        Err(e) => Err(e),
    };
    handle("kanban_update_etiqueta", outcome)
}

#[tauri::command]
pub async fn kanban_delete_etiqueta(
    state: State<'_, AppState>,
    id: Uuid,
    row_version: String,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.delete_etiqueta(id, &row_version).await,
        Err(e) => Err(e),
    };
    handle("kanban_delete_etiqueta", outcome)
}

#[tauri::command]
pub async fn kanban_list_checklist(
    state: State<'_, AppState>,
    tarjeta_id: Uuid,
) -> ApiResult<Vec<KanbanChecklistDto>> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.list_checklist(tarjeta_id).await,
        Err(e) => Err(e),
    };
    handle("kanban_list_checklist", outcome)
}

#[tauri::command]
pub async fn kanban_add_checklist_item(
    state: State<'_, AppState>,
    input: CrearChecklistInput,
) -> ApiResult<KanbanChecklistDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.add_checklist_item(input).await,
        Err(e) => Err(e),
    };
    handle("kanban_add_checklist_item", outcome)
}

#[tauri::command]
pub async fn kanban_update_checklist_item(
    state: State<'_, AppState>,
    id: Uuid,
    input: ActualizarChecklistInput,
) -> ApiResult<KanbanChecklistDto> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.update_checklist_item(id, input).await,
        Err(e) => Err(e),
    };
    handle("kanban_update_checklist_item", outcome)
}

#[tauri::command]
pub async fn kanban_delete_checklist_item(
    state: State<'_, AppState>,
    id: Uuid,
) -> ApiResult<()> {
    let outcome = match state.services() {
        Ok(services) => services.kanban.delete_checklist_item(id).await,
        Err(e) => Err(e),
    };
    handle("kanban_delete_checklist_item", outcome)
}
