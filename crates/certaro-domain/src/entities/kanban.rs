use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::audit::Audit;
use crate::error::DomainError;

/// Prioridad de una tarjeta en el tablero Kanban.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PrioridadTarjeta {
    Baja,
    #[default]
    Normal,
    Alta,
    Urgente,
}

impl PrioridadTarjeta {
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Baja => 0,
            Self::Normal => 1,
            Self::Alta => 2,
            Self::Urgente => 3,
        }
    }

    pub fn from_i32(value: i32) -> Result<Self, DomainError> {
        match value {
            0 => Ok(Self::Baja),
            1 => Ok(Self::Normal),
            2 => Ok(Self::Alta),
            3 => Ok(Self::Urgente),
            other => Err(DomainError::UnknownEnumValue {
                enum_name: "PrioridadTarjeta",
                value: other,
            }),
        }
    }
}

/// Tipo de preset cuando el tablero está sincronizado con entidades de dominio existentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoPresetTablero {
    Trabajos,
    Ordenes,
}

impl TipoPresetTablero {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trabajos => "trabajos",
            Self::Ordenes => "ordenes",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "trabajos" => Some(Self::Trabajos),
            "ordenes" => Some(Self::Ordenes),
            _ => None,
        }
    }
}

/// Tablero Kanban (custom o preset).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanTablero {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub color: Option<String>,
    pub es_preset: bool,
    pub tipo_preset: Option<TipoPresetTablero>,
    pub activo: bool,
    #[serde(flatten)]
    pub audit: Audit,
}

/// Columna de un tablero Kanban.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanColumna {
    pub id: Uuid,
    pub tablero_id: Uuid,
    pub nombre: String,
    pub color: Option<String>,
    pub orden: i32,
    pub limite_wip: Option<i32>,
    /// Para tableros tipo preset, almacena el valor numérico del estado asociado
    /// (e.g. `EstadoTrabajo::EnProceso.as_i32()`).
    pub estado_mapeado: Option<i32>,
    #[serde(flatten)]
    pub audit: Audit,
}

/// Tarjeta del tablero Kanban.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanTarjeta {
    pub id: Uuid,
    pub columna_id: Uuid,
    pub titulo: String,
    pub descripcion: Option<String>,
    pub prioridad: PrioridadTarjeta,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub orden: i32,
    pub trabajo_id: Option<Uuid>,
    pub orden_trabajo_id: Option<Uuid>,
    pub archivada: bool,
    #[serde(flatten)]
    pub audit: Audit,
}

/// Etiqueta / Tag para categorizar tarjetas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanEtiqueta {
    pub id: Uuid,
    pub nombre: String,
    pub color: String,
    #[serde(flatten)]
    pub audit: Audit,
}

/// Relación muchos a muchos entre tarjeta y etiqueta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanTarjetaEtiqueta {
    pub tarjeta_id: Uuid,
    pub etiqueta_id: Uuid,
}

/// Elemento de checklist / subtarea dentro de una tarjeta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanTarjetaChecklist {
    pub id: Uuid,
    pub tarjeta_id: Uuid,
    pub titulo: String,
    pub completada: bool,
    pub orden: i32,
    #[serde(flatten)]
    pub audit: Audit,
}

/// Asignación de empleado/usuario responsable a una tarjeta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KanbanTarjetaAsignado {
    pub tarjeta_id: Uuid,
    pub empleado_id: Option<Uuid>,
    pub usuario_id: Option<Uuid>,
    pub asignado_en: DateTime<Utc>,
}
