//! Contract of the `tipos_movimiento` module. See `docs/11-contratos-tauri.md` §5.11.

use eo_domain::entities::TipoMovimiento;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::AuditDto;
use crate::ports::repositories::{TipoMovimientoConUso, TipoMovimientoFiltro};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TipoMovimientoFiltroDto {
    pub texto: Option<String>,
    pub es_ingreso: Option<bool>,
    pub es_sistema: Option<bool>,
}

impl From<TipoMovimientoFiltroDto> for TipoMovimientoFiltro {
    fn from(dto: TipoMovimientoFiltroDto) -> Self {
        Self {
            texto: dto.texto.filter(|t| !t.trim().is_empty()),
            es_ingreso: dto.es_ingreso,
            es_sistema: dto.es_sistema,
        }
    }
}

/// What the user can type. Note that `esSistema` is absent: a system row is created by the seed,
/// never by the interface.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TipoMovimientoInput {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub es_ingreso: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TipoMovimientoListItem {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub es_ingreso: bool,
    pub es_sistema: bool,
    pub movimientos_count: u64,
    /// False when the row is seeded or flagged as a system row: the interface disables its
    /// delete action instead of letting the user find out by way of an error.
    pub puede_eliminarse: bool,
    pub row_version: String,
}

impl From<TipoMovimientoConUso> for TipoMovimientoListItem {
    fn from(row: TipoMovimientoConUso) -> Self {
        let protegido = row.tipo.es_de_sistema_protegido();
        Self {
            id: row.tipo.id,
            nombre: row.tipo.nombre,
            descripcion: row.tipo.descripcion,
            es_ingreso: row.tipo.es_ingreso,
            es_sistema: row.tipo.es_sistema,
            movimientos_count: row.movimientos_count,
            puede_eliminarse: !protegido && row.movimientos_count == 0,
            row_version: row.tipo.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TipoMovimientoDetalle {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub es_ingreso: bool,
    pub es_sistema: bool,
    pub movimientos_count: u64,
    pub puede_eliminarse: bool,
    pub audit: AuditDto,
}

impl TipoMovimientoDetalle {
    pub fn build(tipo: &TipoMovimiento, movimientos_count: u64) -> Self {
        let protegido = tipo.es_de_sistema_protegido();
        Self {
            id: tipo.id,
            nombre: tipo.nombre.clone(),
            descripcion: tipo.descripcion.clone(),
            es_ingreso: tipo.es_ingreso,
            es_sistema: tipo.es_sistema,
            movimientos_count,
            puede_eliminarse: !protegido && movimientos_count == 0,
            audit: AuditDto::from(&tipo.audit),
        }
    }
}
