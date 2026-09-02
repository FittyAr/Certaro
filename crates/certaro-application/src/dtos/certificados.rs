//! Contract of the `certificados` module. See `docs/11-contratos-tauri.md` §5.5.

use chrono::NaiveDate;
use certaro_domain::{Decimal4, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::AuditDto;
use crate::ports::repositories::{CertificadoConRelaciones, CertificadoFiltro};
use crate::result::AppResult;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificadoFiltroDto {
    pub proyecto_id: Option<Uuid>,
    pub trabajo_id: Option<Uuid>,
    pub cliente_id: Option<Uuid>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
}

impl From<CertificadoFiltroDto> for CertificadoFiltro {
    fn from(dto: CertificadoFiltroDto) -> Self {
        Self {
            proyecto_id: dto.proyecto_id,
            trabajo_id: dto.trabajo_id,
            cliente_id: dto.cliente_id,
            fecha_desde: dto.fecha_desde,
            fecha_hasta: dto.fecha_hasta,
        }
    }
}

/// What the issuing form sends. There is no per-item id list of its own: the percentages come from
/// the order's items, and everything else is computed by the use case from the frozen values.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificadoInput {
    pub orden_trabajo_id: Uuid,
    pub fecha: NaiveDate,
    pub observaciones: Option<String>,
    /// The progress of this certification, per item of the order. An item that is absent, or comes
    /// with zero, is not certified this time.
    pub items: Vec<CertificadoInputItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificadoInputItem {
    pub orden_trabajo_item_id: Uuid,
    pub porcentaje_actual: Decimal4,
}

/// The prefilled form of a new certificate. See `docs/11-contratos-tauri.md` §5.5.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificadoBorrador {
    pub orden_trabajo_id: Uuid,
    pub orden_titulo: String,
    pub numero_sugerido: i32,
    pub trabajo_descripcion: String,
    pub proyecto_nombre: String,
    pub cliente_nombre: String,
    pub ajuste_uocra_porcentaje: Decimal4,
    pub otros_descuentos: Money,
    pub items: Vec<CertificadoBorradorItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificadoBorradorItem {
    pub orden_trabajo_item_id: Uuid,
    pub descripcion: String,
    pub unidad: String,
    pub cantidad: Decimal4,
    pub precio_unitario: Money,
    /// Sum of the percentages of the previous certificates of this item.
    pub porcentaje_acumulado_anterior: Decimal4,
    /// `100 - porcentaje_acumulado_anterior`: the ceiling of what can be certified now.
    pub porcentaje_disponible: Decimal4,
    /// Progress the item currently carries, which is what the form starts with.
    pub porcentaje_actual: Decimal4,
    pub base: Money,
    pub subtotal_acumulado_anterior: Money,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificadoItemDto {
    pub id: Uuid,
    pub orden_trabajo_item_id: Uuid,
    /// Read from the order, not frozen: it is a label, and a corrected typo should show corrected.
    pub descripcion: String,
    pub unidad: String,
    pub cantidad: Decimal4,
    pub precio_unitario: Money,
    pub porcentaje_anterior: Decimal4,
    pub porcentaje_actual: Decimal4,
    pub porcentaje_acumulado: Decimal4,
    pub subtotal_actual: Money,
    pub subtotal_acumulado: Money,
}

impl CertificadoItemDto {
    pub fn build(
        item: &certaro_domain::CertificadoItem,
        descripcion: String,
        unidad: String,
    ) -> AppResult<Self> {
        Ok(Self {
            id: item.id,
            orden_trabajo_item_id: item.orden_trabajo_item_id,
            descripcion,
            unidad,
            cantidad: item.cantidad,
            precio_unitario: item.precio_unitario,
            porcentaje_anterior: item.porcentaje_anterior,
            porcentaje_actual: item.porcentaje_actual,
            porcentaje_acumulado: item.porcentaje_acumulado()?,
            subtotal_actual: item.subtotal_actual,
            subtotal_acumulado: item.subtotal_acumulado,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificadoListItem {
    pub id: Uuid,
    pub numero: i32,
    pub fecha: NaiveDate,
    pub orden_trabajo_id: Uuid,
    pub orden_titulo: String,
    pub trabajo_id: Uuid,
    pub trabajo_descripcion: String,
    pub proyecto_id: Uuid,
    pub proyecto_numero: i32,
    pub proyecto_nombre: String,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub total_certificado: Money,
    pub total_neto: Money,
    pub es_ultimo: bool,
    pub row_version: String,
}

impl From<CertificadoConRelaciones> for CertificadoListItem {
    fn from(row: CertificadoConRelaciones) -> Self {
        Self {
            id: row.certificado.id,
            numero: row.certificado.numero,
            fecha: row.certificado.fecha,
            orden_trabajo_id: row.orden_trabajo_id,
            orden_titulo: row.orden_titulo,
            trabajo_id: row.trabajo_id,
            trabajo_descripcion: row.trabajo_descripcion,
            proyecto_id: row.proyecto_id,
            proyecto_numero: row.proyecto_numero,
            proyecto_nombre: row.proyecto_nombre,
            cliente_id: row.cliente_id,
            cliente_nombre: row.cliente_nombre,
            total_certificado: row.certificado.total_certificado,
            total_neto: row.certificado.total_neto,
            es_ultimo: row.es_ultimo,
            row_version: row.certificado.audit.row_version.to_hex(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificadoDetalle {
    pub id: Uuid,
    pub numero: i32,
    pub fecha: NaiveDate,
    pub observaciones: Option<String>,
    pub orden_trabajo_id: Uuid,
    pub orden_titulo: String,
    pub trabajo_id: Uuid,
    pub trabajo_descripcion: String,
    pub proyecto_id: Uuid,
    pub proyecto_numero: i32,
    pub proyecto_nombre: String,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub total_certificado: Money,
    pub ajuste_uocra: Money,
    pub otros_descuentos: Money,
    pub total_neto: Money,
    pub items: Vec<CertificadoItemDto>,
    /// Only the last certificate of an order can be voided (doc 06 §5.6).
    pub es_ultimo: bool,
    pub audit: AuditDto,
}

impl CertificadoDetalle {
    /// `etiquetas` maps each `orden_trabajo_item_id` to its current description and unit.
    pub fn build(
        row: &CertificadoConRelaciones,
        etiquetas: &std::collections::HashMap<Uuid, (String, String)>,
    ) -> AppResult<Self> {
        let items = row
            .certificado
            .items
            .iter()
            .map(|i| {
                let (descripcion, unidad) = etiquetas
                    .get(&i.orden_trabajo_item_id)
                    .cloned()
                    .unwrap_or_default();
                CertificadoItemDto::build(i, descripcion, unidad)
            })
            .collect::<AppResult<Vec<_>>>()?;
        Ok(Self {
            id: row.certificado.id,
            numero: row.certificado.numero,
            fecha: row.certificado.fecha,
            observaciones: row.certificado.observaciones.clone(),
            orden_trabajo_id: row.orden_trabajo_id,
            orden_titulo: row.orden_titulo.clone(),
            trabajo_id: row.trabajo_id,
            trabajo_descripcion: row.trabajo_descripcion.clone(),
            proyecto_id: row.proyecto_id,
            proyecto_numero: row.proyecto_numero,
            proyecto_nombre: row.proyecto_nombre.clone(),
            cliente_id: row.cliente_id,
            cliente_nombre: row.cliente_nombre.clone(),
            total_certificado: row.certificado.total_certificado,
            ajuste_uocra: row.certificado.ajuste_uocra,
            otros_descuentos: row.certificado.otros_descuentos,
            total_neto: row.certificado.total_neto,
            items,
            es_ultimo: row.es_ultimo,
            audit: AuditDto::from(&row.certificado.audit),
        })
    }
}
