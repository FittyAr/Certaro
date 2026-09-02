//! Contract of the `proyectos` module. See `docs/11-contratos-tauri.md` §5.3.

use certaro_domain::entities::Proyecto;
use certaro_domain::{EstadoProyecto, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::{AuditDto, EstadoInfo};
use crate::ports::repositories::{ProyectoConResumen, ProyectoFiltro};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProyectoFiltroDto {
    pub texto: Option<String>,
    pub cliente_id: Option<Uuid>,
    pub estado: Option<EstadoProyecto>,
    #[serde(default)]
    pub solo_activas: bool,
}

impl From<ProyectoFiltroDto> for ProyectoFiltro {
    fn from(dto: ProyectoFiltroDto) -> Self {
        Self {
            texto: dto.texto.filter(|t| !t.trim().is_empty()),
            cliente_id: dto.cliente_id,
            estado: dto.estado,
            solo_activas: dto.solo_activas,
        }
    }
}

/// The state is absent on purpose: it only ever changes through `proyectos_transition`, so accepting
/// it here would give the form a second, unguarded way in.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProyectoInput {
    pub numero: i32,
    pub nombre: String,
    pub direccion: Option<String>,
    pub localidad: Option<String>,
    pub cliente_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProyectoListItem {
    pub id: Uuid,
    pub numero: i32,
    pub nombre: String,
    pub direccion: Option<String>,
    pub localidad: Option<String>,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub estado: EstadoProyecto,
    pub trabajos_count: u64,
    pub rentabilidad: Money,
    pub puede_eliminarse: bool,
    pub row_version: String,
}

impl From<ProyectoConResumen> for ProyectoListItem {
    fn from(row: ProyectoConResumen) -> Self {
        Self {
            id: row.proyecto.id,
            numero: row.proyecto.numero,
            nombre: row.proyecto.nombre,
            direccion: row.proyecto.direccion,
            localidad: row.proyecto.localidad,
            cliente_id: row.proyecto.cliente_id,
            cliente_nombre: row.cliente_nombre,
            estado: row.proyecto.estado,
            trabajos_count: row.trabajos_count,
            rentabilidad: row.rentabilidad,
            puede_eliminarse: row.trabajos_count == 0,
            row_version: row.proyecto.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProyectoDetalle {
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

impl ProyectoDetalle {
    pub fn build(
        proyecto: &Proyecto,
        cliente_nombre: String,
        trabajos_count: u64,
        rentabilidad: Money,
    ) -> Self {
        Self {
            id: proyecto.id,
            numero: proyecto.numero,
            nombre: proyecto.nombre.clone(),
            direccion: proyecto.direccion.clone(),
            localidad: proyecto.localidad.clone(),
            cliente_id: proyecto.cliente_id,
            cliente_nombre,
            estado: EstadoInfo::build(proyecto.estado, EstadoProyecto::requiere_confirmacion_desde),
            trabajos_count,
            rentabilidad,
            puede_eliminarse: trabajos_count == 0,
            audit: AuditDto::from(&proyecto.audit),
        }
    }
}

impl From<ProyectoConResumen> for ProyectoDetalle {
    fn from(row: ProyectoConResumen) -> Self {
        Self::build(
            &row.proyecto,
            row.cliente_nombre,
            row.trabajos_count,
            row.rentabilidad,
        )
    }
}
