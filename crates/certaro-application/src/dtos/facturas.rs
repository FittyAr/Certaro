//! Contract of the `facturas` module. See `docs/11-contratos-tauri.md` §5.6.

use chrono::NaiveDate;
use certaro_domain::entities::{Factura, PagoFactura};
use certaro_domain::{EstadoFactura, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::{AuditDto, EstadoInfo};
use crate::ports::repositories::{FacturaConResumen, FacturaFiltro};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FacturaFiltroDto {
    pub texto: Option<String>,
    pub cliente_id: Option<Uuid>,
    #[serde(default)]
    pub estados: Vec<EstadoFactura>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
    #[serde(default)]
    pub solo_impagas: bool,
    #[serde(default)]
    pub solo_vencidas: bool,
}

impl From<FacturaFiltroDto> for FacturaFiltro {
    fn from(dto: FacturaFiltroDto) -> Self {
        Self {
            texto: dto.texto.filter(|t| !t.trim().is_empty()),
            cliente_id: dto.cliente_id,
            estados: dto.estados,
            fecha_desde: dto.fecha_desde,
            fecha_hasta: dto.fecha_hasta,
            solo_impagas: dto.solo_impagas,
            solo_vencidas: dto.solo_vencidas,
        }
    }
}

/// `total` is here because the form shows it, but the use case overwrites it with
/// `subtotal + iva`: the validation exists to catch a frontend out of step, not to trust the
/// number that arrives.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FacturaInput {
    pub numero: String,
    pub fecha: NaiveDate,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub cliente_id: Uuid,
    pub subtotal: Money,
    pub iva: Money,
    pub total: Money,
    pub observaciones: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagoFacturaInput {
    pub factura_id: Uuid,
    pub fecha: NaiveDate,
    pub monto: Money,
    pub medio_pago: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagoFacturaItem {
    pub id: Uuid,
    pub factura_id: Uuid,
    pub fecha: NaiveDate,
    pub monto: Money,
    pub medio_pago: String,
    pub row_version: String,
}

impl From<&PagoFactura> for PagoFacturaItem {
    fn from(p: &PagoFactura) -> Self {
        Self {
            id: p.id,
            factura_id: p.factura_id,
            fecha: p.fecha,
            monto: p.monto,
            medio_pago: p.medio_pago.clone(),
            row_version: p.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacturaListItem {
    pub id: Uuid,
    pub numero: String,
    pub fecha: NaiveDate,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub estado: EstadoFactura,
    pub subtotal: Money,
    pub iva: Money,
    pub total: Money,
    pub pagado: Money,
    pub saldo: Money,
    pub dias_mora: i64,
    pub row_version: String,
}

impl FacturaListItem {
    pub fn build(row: FacturaConResumen, hoy: NaiveDate, dias_default: u32) -> Self {
        let dias_mora = row.factura.dias_mora(hoy, dias_default).unwrap_or(0);
        Self {
            id: row.factura.id,
            numero: row.factura.numero,
            fecha: row.factura.fecha,
            fecha_vencimiento: row.factura.fecha_vencimiento,
            cliente_id: row.factura.cliente_id,
            cliente_nombre: row.cliente_nombre,
            estado: row.factura.estado,
            subtotal: row.factura.subtotal,
            iva: row.factura.iva,
            total: row.factura.total,
            pagado: row.pagado,
            saldo: row.saldo,
            dias_mora,
            row_version: row.factura.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacturaDetalle {
    pub id: Uuid,
    pub numero: String,
    pub fecha: NaiveDate,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub estado: EstadoInfo,
    pub subtotal: Money,
    pub iva: Money,
    pub total: Money,
    pub pagado: Money,
    pub saldo: Money,
    pub dias_mora: i64,
    pub observaciones: Option<String>,
    pub pagos: Vec<PagoFacturaItem>,
    /// Whether the current state takes payments at all, so the interface can disable the form
    /// instead of letting the user fill it and be refused.
    pub admite_pagos: bool,
    pub puede_eliminarse: bool,
    pub audit: AuditDto,
}

impl FacturaDetalle {
    pub fn build(
        factura: &Factura,
        cliente_nombre: String,
        puede_eliminarse: bool,
        hoy: NaiveDate,
        dias_default: u32,
    ) -> crate::result::AppResult<Self> {
        let pagado = factura.total_pagado()?;
        let saldo = factura.saldo_pendiente()?;
        Ok(Self {
            id: factura.id,
            numero: factura.numero.clone(),
            fecha: factura.fecha,
            fecha_vencimiento: factura.fecha_vencimiento,
            cliente_id: factura.cliente_id,
            cliente_nombre,
            estado: EstadoInfo::build(factura.estado, |_, _| false),
            subtotal: factura.subtotal,
            iva: factura.iva,
            total: factura.total,
            pagado,
            saldo,
            dias_mora: factura.dias_mora(hoy, dias_default)?,
            observaciones: factura.observaciones.clone(),
            pagos: factura
                .pagos
                .iter()
                .filter(|p| !p.audit.is_deleted)
                .map(PagoFacturaItem::from)
                .collect(),
            admite_pagos: factura.estado.admite_pagos(),
            puede_eliminarse,
            audit: AuditDto::from(&factura.audit),
        })
    }
}
