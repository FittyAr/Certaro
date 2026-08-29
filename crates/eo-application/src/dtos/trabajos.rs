//! Contract of the `trabajos` module. See `docs/11-contratos-tauri.md` §5.4.

use chrono::NaiveDate;
use eo_domain::{EstadoTrabajo, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::{AuditDto, EstadoInfo};
use crate::ports::repositories::{TrabajoConRelaciones, TrabajoFiltro};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrabajoFiltroDto {
    pub texto: Option<String>,
    pub obra_id: Option<Uuid>,
    /// Resolved through the site: the job carries no customer of its own, and the legacy
    /// denormalised column is exactly what made this filter return the wrong rows.
    pub cliente_id: Option<Uuid>,
    pub estado: Option<EstadoTrabajo>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
}

impl From<TrabajoFiltroDto> for TrabajoFiltro {
    fn from(dto: TrabajoFiltroDto) -> Self {
        Self {
            texto: dto.texto.filter(|t| !t.trim().is_empty()),
            obra_id: dto.obra_id,
            cliente_id: dto.cliente_id,
            estado: dto.estado,
            fecha_desde: dto.fecha_desde,
            fecha_hasta: dto.fecha_hasta,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrabajoInput {
    pub obra_id: Uuid,
    pub descripcion: String,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: Option<NaiveDate>,
    pub presupuesto: Money,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrabajoListItem {
    pub id: Uuid,
    pub obra_id: Uuid,
    pub obra_numero: i32,
    pub obra_nombre: String,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub descripcion: String,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: Option<NaiveDate>,
    pub presupuesto: Money,
    pub estado: EstadoTrabajo,
    pub row_version: String,
}

impl From<TrabajoConRelaciones> for TrabajoListItem {
    fn from(row: TrabajoConRelaciones) -> Self {
        Self {
            id: row.trabajo.id,
            obra_id: row.trabajo.obra_id,
            obra_numero: row.obra_numero,
            obra_nombre: row.obra_nombre,
            cliente_id: row.cliente_id,
            cliente_nombre: row.cliente_nombre,
            descripcion: row.trabajo.descripcion,
            fecha_inicio: row.trabajo.fecha_inicio,
            fecha_fin: row.trabajo.fecha_fin,
            presupuesto: row.trabajo.presupuesto,
            estado: row.trabajo.estado,
            row_version: row.trabajo.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrabajoDetalle {
    pub id: Uuid,
    pub obra_id: Uuid,
    pub obra_numero: i32,
    pub obra_nombre: String,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub descripcion: String,
    pub fecha_inicio: NaiveDate,
    pub fecha_fin: Option<NaiveDate>,
    pub presupuesto: Money,
    pub estado: EstadoInfo,
    pub puede_eliminarse: bool,
    pub audit: AuditDto,
}

impl TrabajoDetalle {
    pub fn build(row: &TrabajoConRelaciones, puede_eliminarse: bool) -> Self {
        Self {
            id: row.trabajo.id,
            obra_id: row.trabajo.obra_id,
            obra_numero: row.obra_numero,
            obra_nombre: row.obra_nombre.clone(),
            cliente_id: row.cliente_id,
            cliente_nombre: row.cliente_nombre.clone(),
            descripcion: row.trabajo.descripcion.clone(),
            fecha_inicio: row.trabajo.fecha_inicio,
            fecha_fin: row.trabajo.fecha_fin,
            presupuesto: row.trabajo.presupuesto,
            estado: EstadoInfo::build(row.trabajo.estado, |desde, hasta| {
                desde.requiere_confirmacion_desde(hasta)
            }),
            puede_eliminarse,
            audit: AuditDto::from(&row.trabajo.audit),
        }
    }
}
