//! Contract of the `clientes` module. See `docs/11-contratos-tauri.md` §5.2.

use chrono::NaiveDate;
use eo_domain::entities::{Cliente, ClienteContacto};
use eo_domain::{EstadoFactura, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::AuditDto;
use crate::ports::repositories::{ClienteConResumen, ClienteFiltro};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClienteFiltroDto {
    pub texto: Option<String>,
    pub condicion_iva: Option<String>,
    #[serde(default)]
    pub solo_con_deuda: bool,
}

impl From<ClienteFiltroDto> for ClienteFiltro {
    fn from(dto: ClienteFiltroDto) -> Self {
        Self {
            texto: dto.texto.filter(|t| !t.trim().is_empty()),
            condicion_iva: dto.condicion_iva.filter(|t| !t.trim().is_empty()),
            solo_con_deuda: dto.solo_con_deuda,
        }
    }
}

/// A customer and its contacts arrive together: they are one aggregate and are saved in one
/// transaction, so there is no separate contact command to get out of step with this one.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClienteInput {
    pub nombre: String,
    pub cuit: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub condicion_iva: Option<String>,
    #[serde(default)]
    pub contactos: Vec<ClienteContactoInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClienteContactoInput {
    /// Absent on a row the user just added; present on one being edited.
    pub id: Option<Uuid>,
    pub etiqueta: String,
    pub email: String,
    pub nombre: Option<String>,
    pub telefono: Option<String>,
    #[serde(default)]
    pub es_principal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClienteContactoDto {
    pub id: Uuid,
    pub etiqueta: String,
    pub email: String,
    pub nombre: Option<String>,
    pub telefono: Option<String>,
    pub es_principal: bool,
}

impl From<&ClienteContacto> for ClienteContactoDto {
    fn from(c: &ClienteContacto) -> Self {
        Self {
            id: c.id,
            etiqueta: c.etiqueta.clone(),
            email: c.email.clone(),
            nombre: c.nombre.clone(),
            telefono: c.telefono.clone(),
            es_principal: c.es_principal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClienteListItem {
    pub id: Uuid,
    pub nombre: String,
    pub cuit: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub condicion_iva: Option<String>,
    pub obras_count: u64,
    pub facturas_count: u64,
    pub deuda: Money,
    pub puede_eliminarse: bool,
    pub row_version: String,
}

impl From<ClienteConResumen> for ClienteListItem {
    fn from(row: ClienteConResumen) -> Self {
        Self {
            id: row.cliente.id,
            nombre: row.cliente.nombre,
            cuit: row.cliente.cuit,
            telefono: row.cliente.telefono,
            email: row.cliente.email,
            condicion_iva: row.cliente.condicion_iva,
            obras_count: row.obras_count,
            facturas_count: row.facturas_count,
            deuda: row.deuda,
            puede_eliminarse: row.obras_count == 0 && row.facturas_count == 0,
            row_version: row.cliente.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClienteDetalle {
    pub id: Uuid,
    pub nombre: String,
    pub cuit: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub condicion_iva: Option<String>,
    pub contactos: Vec<ClienteContactoDto>,
    pub obras_count: u64,
    pub facturas_count: u64,
    pub puede_eliminarse: bool,
    pub audit: AuditDto,
}

impl ClienteDetalle {
    pub fn build(cliente: &Cliente, obras_count: u64, facturas_count: u64) -> Self {
        Self {
            id: cliente.id,
            nombre: cliente.nombre.clone(),
            cuit: cliente.cuit.clone(),
            direccion: cliente.direccion.clone(),
            telefono: cliente.telefono.clone(),
            email: cliente.email.clone(),
            condicion_iva: cliente.condicion_iva.clone(),
            contactos: cliente
                .contactos
                .iter()
                .filter(|c| !c.audit.is_deleted)
                .map(ClienteContactoDto::from)
                .collect(),
            obras_count,
            facturas_count,
            puede_eliminarse: obras_count == 0 && facturas_count == 0,
            audit: AuditDto::from(&cliente.audit),
        }
    }
}

/// The account statement of one customer. See `docs/06-casos-de-uso-y-formulas.md` §4.5.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CuentaCorriente {
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub total_facturado: Money,
    pub total_pagado: Money,
    pub saldo: Money,
    pub facturas: Vec<CuentaCorrienteFactura>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CuentaCorrienteFactura {
    pub id: Uuid,
    pub numero: String,
    pub fecha: NaiveDate,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub estado: EstadoFactura,
    pub total: Money,
    pub pagado: Money,
    pub saldo: Money,
    pub dias_mora: i64,
}

/// Debt split into age buckets. See `docs/06-casos-de-uso-y-formulas.md` §4.6.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiguedadDeuda {
    pub fecha_corte: NaiveDate,
    pub total: Money,
    pub bucket0a30: Money,
    pub bucket31a60: Money,
    pub bucket61a90: Money,
    pub bucket_mas90: Money,
    pub detalle: Vec<AntiguedadDeudaCliente>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AntiguedadDeudaCliente {
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub total: Money,
    pub bucket0a30: Money,
    pub bucket31a60: Money,
    pub bucket61a90: Money,
    pub bucket_mas90: Money,
}
