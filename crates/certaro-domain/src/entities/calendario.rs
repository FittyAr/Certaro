use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::audit::Audit;
use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TipoRecurso {
    Empleado,
    Vehiculo,
    Herramienta,
    Proyecto,
}

impl TipoRecurso {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Empleado => "empleado",
            Self::Vehiculo => "vehiculo",
            Self::Herramienta => "herramienta",
            Self::Proyecto => "proyecto",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw.to_lowercase().as_str() {
            "empleado" => Ok(Self::Empleado),
            "vehiculo" => Ok(Self::Vehiculo),
            "herramienta" => Ok(Self::Herramienta),
            "proyecto" => Ok(Self::Proyecto),
            _ => Err(DomainError::InvariantViolated("tipo de recurso invalido")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TipoEvento {
    Trabajo,
    Reunion,
    Mantenimiento,
    Entrega,
    Otro,
}

impl TipoEvento {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trabajo => "trabajo",
            Self::Reunion => "reunion",
            Self::Mantenimiento => "mantenimiento",
            Self::Entrega => "entrega",
            Self::Otro => "otro",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw.to_lowercase().as_str() {
            "trabajo" => Ok(Self::Trabajo),
            "reunion" => Ok(Self::Reunion),
            "mantenimiento" => Ok(Self::Mantenimiento),
            "entrega" => Ok(Self::Entrega),
            "otro" => Ok(Self::Otro),
            _ => Err(DomainError::InvariantViolated("tipo de evento invalido")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarioGrupoRecurso {
    pub id: Uuid,
    pub nombre: String,
    pub color: Option<String>,
    #[serde(flatten)]
    pub audit: Audit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarioRecurso {
    pub id: Uuid,
    pub grupo_id: Option<Uuid>,
    pub nombre: String,
    pub tipo: TipoRecurso,
    pub empleado_id: Option<Uuid>,
    pub color: Option<String>,
    pub activo: bool,
    #[serde(flatten)]
    pub audit: Audit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarioEvento {
    pub id: Uuid,
    pub titulo: String,
    pub descripcion: Option<String>,
    pub tipo: TipoEvento,
    pub inicio: DateTime<Utc>,
    pub fin: DateTime<Utc>,
    pub todo_el_dia: bool,
    pub color: Option<String>,
    pub trabajo_id: Option<Uuid>,
    pub kanban_tarjeta_id: Option<Uuid>,
    #[serde(flatten)]
    pub audit: Audit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarioEventoRecurso {
    pub evento_id: Uuid,
    pub recurso_id: Uuid,
}
