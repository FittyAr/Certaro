use certaro_domain::entities::calendario::{TipoEvento, TipoRecurso};
use certaro_domain::RowVersion;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarioGrupoRecursoDto {
    pub id: Uuid,
    pub nombre: String,
    pub color: Option<String>,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarioRecursoDto {
    pub id: Uuid,
    pub grupo_id: Option<Uuid>,
    pub grupo_nombre: Option<String>,
    pub nombre: String,
    pub tipo: TipoRecurso,
    pub empleado_id: Option<Uuid>,
    pub color: Option<String>,
    pub activo: bool,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarioEventoDto {
    pub id: Uuid,
    pub titulo: String,
    pub descripcion: Option<String>,
    pub tipo: TipoEvento,
    pub inicio: String,
    pub fin: String,
    pub todo_el_dia: bool,
    pub color: Option<String>,
    pub trabajo_id: Option<Uuid>,
    pub kanban_tarjeta_id: Option<Uuid>,
    pub recursos: Vec<CalendarioRecursoDto>,
    pub es_virtual: bool,
    pub row_version: RowVersion,
}

// --- Inputs ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearGrupoRecursoInput {
    pub nombre: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarGrupoRecursoInput {
    pub nombre: String,
    pub color: Option<String>,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearRecursoInput {
    pub grupo_id: Option<Uuid>,
    pub nombre: String,
    pub tipo: TipoRecurso,
    pub empleado_id: Option<Uuid>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarRecursoInput {
    pub grupo_id: Option<Uuid>,
    pub nombre: String,
    pub tipo: TipoRecurso,
    pub empleado_id: Option<Uuid>,
    pub color: Option<String>,
    pub activo: bool,
    pub row_version: RowVersion,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearEventoInput {
    pub titulo: String,
    pub descripcion: Option<String>,
    pub tipo: TipoEvento,
    pub inicio: String,
    pub fin: String,
    pub todo_el_dia: bool,
    pub color: Option<String>,
    pub trabajo_id: Option<Uuid>,
    pub kanban_tarjeta_id: Option<Uuid>,
    pub recurso_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarEventoInput {
    pub titulo: String,
    pub descripcion: Option<String>,
    pub tipo: TipoEvento,
    pub inicio: String,
    pub fin: String,
    pub todo_el_dia: bool,
    pub color: Option<String>,
    pub trabajo_id: Option<Uuid>,
    pub recurso_ids: Option<Vec<Uuid>>,
    pub row_version: RowVersion,
}
