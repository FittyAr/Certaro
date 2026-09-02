#![allow(non_snake_case)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use certaro_domain::entities::kanban::{
    KanbanColumna, KanbanEtiqueta, KanbanTablero, KanbanTarjetaChecklist, PrioridadTarjeta,
    TipoPresetTablero,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanTableroDto {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color: Option<String>,
    pub esPreset: bool,
    pub tipoPreset: Option<TipoPresetTablero>,
    pub activo: bool,
    pub rowVersion: String,
}

impl From<KanbanTablero> for KanbanTableroDto {
    fn from(t: KanbanTablero) -> Self {
        Self {
            id: t.id,
            nombre: t.nombre,
            descripcion: t.descripcion,
            color: t.color,
            esPreset: t.es_preset,
            tipoPreset: t.tipo_preset,
            activo: t.activo,
            rowVersion: t.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanColumnaDto {
    pub id: Uuid,
    pub tableroId: Uuid,
    pub nombre: String,
    pub color: Option<String>,
    pub orden: i32,
    pub limiteWip: Option<i32>,
    pub estadoMapeado: Option<i32>,
    pub rowVersion: String,
}

impl From<KanbanColumna> for KanbanColumnaDto {
    fn from(c: KanbanColumna) -> Self {
        Self {
            id: c.id,
            tableroId: c.tablero_id,
            nombre: c.nombre,
            color: c.color,
            orden: c.orden,
            limiteWip: c.limite_wip,
            estadoMapeado: c.estado_mapeado,
            rowVersion: c.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanEtiquetaDto {
    pub id: Uuid,
    pub nombre: String,
    pub color: String,
    pub rowVersion: String,
}

impl From<KanbanEtiqueta> for KanbanEtiquetaDto {
    fn from(e: KanbanEtiqueta) -> Self {
        Self {
            id: e.id,
            nombre: e.nombre,
            color: e.color,
            rowVersion: e.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanChecklistDto {
    pub id: Uuid,
    pub tarjetaId: Uuid,
    pub titulo: String,
    pub completada: bool,
    pub orden: i32,
    pub rowVersion: String,
}

impl From<KanbanTarjetaChecklist> for KanbanChecklistDto {
    fn from(c: KanbanTarjetaChecklist) -> Self {
        Self {
            id: c.id,
            tarjetaId: c.tarjeta_id,
            titulo: c.titulo,
            completada: c.completada,
            orden: c.orden,
            rowVersion: c.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanTarjetaDto {
    pub id: Uuid,
    pub columnaId: Uuid,
    pub titulo: String,
    pub descripcion: Option<String>,
    pub prioridad: PrioridadTarjeta,
    pub fechaVencimiento: Option<NaiveDate>,
    pub orden: i32,
    pub trabajoId: Option<Uuid>,
    pub ordenTrabajoId: Option<Uuid>,
    pub archivada: bool,
    pub rowVersion: String,
    pub etiquetas: Vec<KanbanEtiquetaDto>,
    pub totalChecklist: usize,
    pub completadasChecklist: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanTableroDetalleDto {
    pub tablero: KanbanTableroDto,
    pub columnas: Vec<KanbanColumnaDto>,
    pub tarjetas: Vec<KanbanTarjetaDto>,
    pub etiquetas: Vec<KanbanEtiquetaDto>,
}

// --- Inputs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearTableroInput {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarTableroInput {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color: Option<String>,
    pub activo: bool,
    pub rowVersion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearColumnaInput {
    pub tableroId: Uuid,
    pub nombre: String,
    pub color: Option<String>,
    pub limiteWip: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarColumnaInput {
    pub nombre: String,
    pub color: Option<String>,
    pub orden: i32,
    pub limiteWip: Option<i32>,
    pub rowVersion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearTarjetaInput {
    pub columnaId: Uuid,
    pub titulo: String,
    pub descripcion: Option<String>,
    pub prioridad: PrioridadTarjeta,
    pub fechaVencimiento: Option<NaiveDate>,
    pub trabajoId: Option<Uuid>,
    pub ordenTrabajoId: Option<Uuid>,
    pub etiquetaIds: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarTarjetaInput {
    pub titulo: String,
    pub descripcion: Option<String>,
    pub prioridad: PrioridadTarjeta,
    pub fechaVencimiento: Option<NaiveDate>,
    pub etiquetaIds: Option<Vec<Uuid>>,
    pub rowVersion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoverTarjetaInput {
    pub tarjetaId: Uuid,
    pub nuevaColumnaId: Uuid,
    pub nuevoOrden: i32,
    pub rowVersion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearEtiquetaInput {
    pub nombre: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarEtiquetaInput {
    pub nombre: String,
    pub color: String,
    pub rowVersion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearChecklistInput {
    pub tarjetaId: Uuid,
    pub titulo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizarChecklistInput {
    pub titulo: String,
    pub completada: bool,
    pub orden: i32,
    pub rowVersion: String,
}
