//! Contract of the `obras` module. See `docs/11-contratos-tauri.md` §5.3.

use eo_domain::entities::Obra;
use eo_domain::{EstadoObra, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::{AuditDto, EstadoInfo};
use crate::ports::repositories::{ObraConResumen, ObraFiltro};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObraFiltroDto {
    pub texto: Option<String>,
    pub cliente_id: Option<Uuid>,
    pub estado: Option<EstadoObra>,
    #[serde(default)]
    pub solo_activas: bool,
}

impl From<ObraFiltroDto> for ObraFiltro {
    fn from(dto: ObraFiltroDto) -> Self {
        Self {
            texto: dto.texto.filter(|t| !t.trim().is_empty()),
            cliente_id: dto.cliente_id,
            estado: dto.estado,
            solo_activas: dto.solo_activas,
        }
    }
}

/// The state is absent on purpose: it only ever changes through `obras_transition`, so accepting
/// it here would give the form a second, unguarded way in.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObraInput {
    pub numero: i32,
    pub nombre: String,
    pub direccion: Option<String>,
    pub localidad: Option<String>,
    pub cliente_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObraListItem {
    pub id: Uuid,
    pub numero: i32,
    pub nombre: String,
    pub direccion: Option<String>,
    pub localidad: Option<String>,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub estado: EstadoObra,
    pub trabajos_count: u64,
    pub rentabilidad: Money,
    pub puede_eliminarse: bool,
    pub row_version: String,
}

impl From<ObraConResumen> for ObraListItem {
    fn from(row: ObraConResumen) -> Self {
        Self {
            id: row.obra.id,
            numero: row.obra.numero,
            nombre: row.obra.nombre,
            direccion: row.obra.direccion,
            localidad: row.obra.localidad,
            cliente_id: row.obra.cliente_id,
            cliente_nombre: row.cliente_nombre,
            estado: row.obra.estado,
            trabajos_count: row.trabajos_count,
            rentabilidad: row.rentabilidad,
            puede_eliminarse: row.trabajos_count == 0,
            row_version: row.obra.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObraDetalle {
    pub id: Uuid,
    pub numero: i32,
    pub nombre: String,
    pub direccion: Option<String>,
    pub localidad: Option<String>,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub estado: EstadoInfo,
    pub trabajos_count: u64,
    pub rentabilidad: Money,
    pub puede_eliminarse: bool,
    pub audit: AuditDto,
}

impl ObraDetalle {
    pub fn build(
        obra: &Obra,
        cliente_nombre: String,
        trabajos_count: u64,
        rentabilidad: Money,
    ) -> Self {
        Self {
            id: obra.id,
            numero: obra.numero,
            nombre: obra.nombre.clone(),
            direccion: obra.direccion.clone(),
            localidad: obra.localidad.clone(),
            cliente_id: obra.cliente_id,
            cliente_nombre,
            estado: EstadoInfo::build(obra.estado, EstadoObra::requiere_confirmacion_desde),
            trabajos_count,
            rentabilidad,
            puede_eliminarse: trabajos_count == 0,
            audit: AuditDto::from(&obra.audit),
        }
    }
}

impl From<ObraConResumen> for ObraDetalle {
    fn from(row: ObraConResumen) -> Self {
        Self::build(
            &row.obra,
            row.cliente_nombre,
            row.trabajos_count,
            row.rentabilidad,
        )
    }
}
