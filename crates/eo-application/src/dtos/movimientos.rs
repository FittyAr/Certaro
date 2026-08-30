//! Contract of the `movimientos` module. See `docs/11-contratos-tauri.md` §5.1.

use chrono::{DateTime, NaiveDate, Utc};
use eo_domain::{Decimal4, Moneda, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::common::AuditDto;
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{MovimientoConRelaciones, MovimientoFiltro, MovimientoResumen};
use crate::result::AppResult;

/// Serialisable as well as deserialisable: the JSON export echoes the filter it applied, so the
/// file says what it is showing (doc 12 §2.5).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoFiltroDto {
    pub concepto: Option<String>,
    pub tipo_movimiento_id: Option<Uuid>,
    pub categoria_id: Option<Uuid>,
    pub cliente_id: Option<Uuid>,
    pub trabajo_id: Option<Uuid>,
    pub empleado_id: Option<Uuid>,
    pub factura_id: Option<Uuid>,
    pub moneda: Option<Moneda>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
    pub monto_min: Option<Money>,
    pub monto_max: Option<Money>,
}

impl From<MovimientoFiltroDto> for MovimientoFiltro {
    fn from(dto: MovimientoFiltroDto) -> Self {
        Self {
            // A filter of only spaces is no filter: it would otherwise match nothing and look
            // like an empty database.
            concepto: dto.concepto.filter(|c| !c.trim().is_empty()),
            tipo_movimiento_id: dto.tipo_movimiento_id,
            categoria_id: dto.categoria_id,
            cliente_id: dto.cliente_id,
            trabajo_id: dto.trabajo_id,
            empleado_id: dto.empleado_id,
            factura_id: dto.factura_id,
            moneda: dto.moneda,
            fecha_desde: dto.fecha_desde,
            fecha_hasta: dto.fecha_hasta,
            monto_min: dto.monto_min,
            monto_max: dto.monto_max,
        }
    }
}

/// What the form sends. `total` is absent on purpose: it is `monto * cantidad` and is derived on
/// every read, never accepted from the client.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoInput {
    pub fecha: DateTime<Utc>,
    pub concepto: String,
    pub monto: Money,
    pub cantidad: Decimal4,
    pub tipo_movimiento_id: Uuid,
    #[serde(default)]
    pub moneda: Moneda,
    pub cotizacion_aplicada: Option<Money>,
    pub tipo_concepto_pago_id: Option<Uuid>,
    pub categoria_id: Option<Uuid>,
    pub cliente_id: Option<Uuid>,
    pub trabajo_id: Option<Uuid>,
    pub empleado_id: Option<Uuid>,
    pub factura_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoListItem {
    pub id: Uuid,
    pub fecha: DateTime<Utc>,
    pub concepto: String,
    pub monto: Money,
    pub cantidad: Decimal4,
    /// Derived, never stored (INV-01).
    pub total: Money,
    pub moneda: Moneda,
    pub cotizacion_aplicada: Option<Money>,
    pub tipo_movimiento_id: Uuid,
    pub tipo_movimiento_nombre: String,
    pub es_ingreso: bool,
    pub categoria_id: Option<Uuid>,
    pub categoria_nombre: Option<String>,
    pub categoria_color: Option<String>,
    pub cliente_id: Option<Uuid>,
    pub cliente_nombre: Option<String>,
    pub trabajo_id: Option<Uuid>,
    pub trabajo_descripcion: Option<String>,
    /// The site of the job, resolved through it: a movement never points at a site directly.
    pub obra_nombre: Option<String>,
    pub empleado_id: Option<Uuid>,
    pub factura_id: Option<Uuid>,
    pub tipo_concepto_pago_id: Option<Uuid>,
    pub bloqueado_por_liquidacion: bool,
    pub row_version: String,
}

impl TryFrom<MovimientoConRelaciones> for MovimientoListItem {
    type Error = AppError;

    fn try_from(row: MovimientoConRelaciones) -> Result<Self, Self::Error> {
        let total = row.movimiento.total().map_err(AppError::from)?;
        let m = row.movimiento;
        Ok(Self {
            id: m.id,
            fecha: m.fecha,
            concepto: m.concepto,
            monto: m.monto,
            cantidad: m.cantidad,
            total,
            moneda: m.moneda,
            cotizacion_aplicada: m.cotizacion_aplicada,
            tipo_movimiento_id: m.tipo_movimiento_id,
            tipo_movimiento_nombre: row.tipo_movimiento_nombre,
            es_ingreso: row.es_ingreso,
            categoria_id: m.categoria_id,
            categoria_nombre: row.categoria_nombre,
            categoria_color: row.categoria_color,
            cliente_id: m.cliente_id,
            cliente_nombre: row.cliente_nombre,
            trabajo_id: m.trabajo_id,
            trabajo_descripcion: row.trabajo_descripcion,
            obra_nombre: row.obra_nombre,
            empleado_id: m.empleado_id,
            factura_id: m.factura_id,
            tipo_concepto_pago_id: m.tipo_concepto_pago_id,
            bloqueado_por_liquidacion: row.bloqueado_por_liquidacion,
            row_version: m.audit.row_version.to_hex(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoDetalle {
    #[serde(flatten)]
    pub item: MovimientoListItem,
    pub audit: AuditDto,
}

impl TryFrom<MovimientoConRelaciones> for MovimientoDetalle {
    type Error = AppError;

    fn try_from(row: MovimientoConRelaciones) -> Result<Self, Self::Error> {
        let audit = AuditDto::from(&row.movimiento.audit);
        Ok(Self {
            item: MovimientoListItem::try_from(row)?,
            audit,
        })
    }
}

/// Income, expenses and balance of the **whole filter**, which is what the screen shows under the
/// table. Summing the visible page instead would give a number that changes with the paging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoResumenDto {
    pub total_ingresos: Money,
    pub total_gastos: Money,
    pub balance: Money,
    pub cantidad: u64,
}

impl From<MovimientoResumen> for MovimientoResumenDto {
    fn from(r: MovimientoResumen) -> Self {
        Self {
            total_ingresos: r.total_ingresos,
            total_gastos: r.total_gastos,
            balance: r.balance,
            cantidad: r.cantidad,
        }
    }
}

/// A page plus the summary, so the screen gets both in one round trip.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoListResult {
    #[serde(flatten)]
    pub page: PagedResult<MovimientoListItem>,
    pub resumen: MovimientoResumenDto,
}

impl MovimientoListResult {
    pub fn build(
        page: PagedResult<MovimientoConRelaciones>,
        resumen: MovimientoResumen,
    ) -> AppResult<Self> {
        let page = page.try_map(MovimientoListItem::try_from)?;
        Ok(Self {
            page,
            resumen: resumen.into(),
        })
    }
}
