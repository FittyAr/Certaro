//! Contract of the `ordenes_trabajo` module. See `docs/11-contratos-tauri.md` §5.4.

use chrono::NaiveDate;
use certaro_domain::{Decimal4, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::AuditDto;
use crate::ports::repositories::OrdenTrabajoConRelaciones;
use crate::result::AppResult;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdenTrabajoInput {
    pub trabajo_id: Uuid,
    pub titulo: String,
    pub fecha: NaiveDate,
    pub observaciones: Option<String>,
    pub ajuste_uocra_porcentaje: Decimal4,
    pub otros_descuentos: Money,
    /// The whole sheet, in the order it is printed. Items are part of the aggregate: what is not
    /// in this list is deleted, unless it has already been certified.
    pub items: Vec<OrdenTrabajoItemInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdenTrabajoItemInput {
    /// Absent on a new line. Present, and belonging to this order, on an edited one.
    pub id: Option<Uuid>,
    pub descripcion: String,
    pub unidad: String,
    pub cantidad: Decimal4,
    pub precio_unitario: Money,
    /// Progress of the certificate being prepared. `porcentaje_anterior` is not in the input: it
    /// is history, written only by issuing or voiding a certificate.
    pub porcentaje_actual: Decimal4,
    pub ejecutado: bool,
    pub nota: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdenTrabajoItemDto {
    pub id: Uuid,
    pub descripcion: String,
    pub unidad: String,
    pub cantidad: Decimal4,
    pub precio_unitario: Money,
    pub porcentaje_anterior: Decimal4,
    pub porcentaje_actual: Decimal4,
    pub porcentaje_acumulado: Decimal4,
    pub porcentaje_pendiente: Decimal4,
    /// Full value of the line, at 100 %.
    pub base: Money,
    pub subtotal_actual: Money,
    pub subtotal_acumulado: Money,
    pub ejecutado: bool,
    pub nota: Option<String>,
    pub orden: i32,
    /// True when some certificate already includes this line, which is what forbids removing it.
    pub certificado: bool,
}

impl OrdenTrabajoItemDto {
    pub fn build(item: &certaro_domain::OrdenTrabajoItem, certificado: bool) -> AppResult<Self> {
        let (subtotal_actual, subtotal_acumulado) = item.subtotales()?;
        Ok(Self {
            id: item.id,
            descripcion: item.descripcion.clone(),
            unidad: item.unidad.clone(),
            cantidad: item.cantidad,
            precio_unitario: item.precio_unitario,
            porcentaje_anterior: item.porcentaje_anterior,
            porcentaje_actual: item.porcentaje_actual,
            porcentaje_acumulado: item.porcentaje_acumulado()?,
            porcentaje_pendiente: item.porcentaje_pendiente()?,
            base: item.base()?,
            subtotal_actual,
            subtotal_acumulado,
            ejecutado: item.ejecutado,
            nota: item.nota.clone(),
            orden: item.orden,
            certificado,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdenTrabajoListItem {
    pub id: Uuid,
    pub trabajo_id: Uuid,
    pub titulo: String,
    pub numero_certificado: Option<String>,
    pub fecha: NaiveDate,
    pub items_count: usize,
    pub total_presupuestado: Money,
    /// What would be certified right now, before the discounts.
    pub total_certificado: Money,
    pub certificados_count: u64,
    pub row_version: String,
}

impl OrdenTrabajoListItem {
    pub fn build(row: &OrdenTrabajoConRelaciones) -> AppResult<Self> {
        Ok(Self {
            id: row.orden.id,
            trabajo_id: row.orden.trabajo_id,
            titulo: row.orden.titulo.clone(),
            numero_certificado: row.orden.numero_certificado.clone(),
            fecha: row.orden.fecha,
            items_count: row.orden.items.len(),
            total_presupuestado: row.orden.total_presupuestado()?,
            total_certificado: row.orden.total_certificado()?,
            certificados_count: row.certificados_count,
            row_version: row.orden.audit.row_version.to_hex(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdenTrabajoDetalle {
    pub id: Uuid,
    pub trabajo_id: Uuid,
    pub trabajo_descripcion: String,
    pub proyecto_id: Uuid,
    pub proyecto_numero: i32,
    pub proyecto_nombre: String,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub titulo: String,
    pub numero_certificado: Option<String>,
    pub fecha: NaiveDate,
    pub observaciones: Option<String>,
    pub ajuste_uocra_porcentaje: Decimal4,
    pub otros_descuentos: Money,
    pub items: Vec<OrdenTrabajoItemDto>,
    pub total_presupuestado: Money,
    pub total_certificado: Money,
    /// The percentage above, resolved into the amount it discounts.
    pub ajuste_uocra: Money,
    pub total_neto: Money,
    pub certificados_count: u64,
    /// False once anything has been certified: the history would lose its anchor.
    pub puede_eliminarse: bool,
    pub audit: AuditDto,
}

impl OrdenTrabajoDetalle {
    pub fn build(row: &OrdenTrabajoConRelaciones, certificados: &[Uuid]) -> AppResult<Self> {
        let items = row
            .orden
            .items
            .iter()
            .map(|i| OrdenTrabajoItemDto::build(i, certificados.contains(&i.id)))
            .collect::<AppResult<Vec<_>>>()?;
        Ok(Self {
            id: row.orden.id,
            trabajo_id: row.orden.trabajo_id,
            trabajo_descripcion: row.trabajo_descripcion.clone(),
            proyecto_id: row.proyecto_id,
            proyecto_numero: row.proyecto_numero,
            proyecto_nombre: row.proyecto_nombre.clone(),
            cliente_id: row.cliente_id,
            cliente_nombre: row.cliente_nombre.clone(),
            titulo: row.orden.titulo.clone(),
            numero_certificado: row.orden.numero_certificado.clone(),
            fecha: row.orden.fecha,
            observaciones: row.orden.observaciones.clone(),
            ajuste_uocra_porcentaje: row.orden.ajuste_uocra_porcentaje,
            otros_descuentos: row.orden.otros_descuentos,
            items,
            total_presupuestado: row.orden.total_presupuestado()?,
            total_certificado: row.orden.total_certificado()?,
            ajuste_uocra: row.orden.ajuste_uocra()?,
            total_neto: row.orden.total_neto()?,
            certificados_count: row.certificados_count,
            puede_eliminarse: row.certificados_count == 0,
            audit: AuditDto::from(&row.orden.audit),
        })
    }
}
