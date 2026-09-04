use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;
use certaro_domain::entities::*;
use certaro_domain::{Decimal4, EstadoFactura, EstadoProyecto, EstadoTrabajo, Moneda, Money, RowVersion};
use crate::paging::{PageRequest, PagedResult};
use crate::result::AppResult;
use super::common::*;
use super::movimientos::MovimientoResumen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotalPorNombre {
    /// Present when the group is an entity the screen can navigate to.
    pub id: Option<Uuid>,
    pub nombre: String,
    pub total: Money,
}

/// One month of the yearly series. Both signs come back together so the chart needs one query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotalMensual {
    /// 1 to 12.
    pub mes: u32,
    pub ingresos: Money,
    pub gastos: Money,
}

/// Profitability of a site or of a job. `etiqueta` is the name the ranking prints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RentabilidadFila {
    pub id: Uuid,
    pub etiqueta: String,
    /// Name of the site the job belongs to; empty when the row *is* a site.
    pub contexto: String,
    pub ingresos: Money,
    pub gastos: Money,
    pub rentabilidad: Money,
}

/// An invoice with an outstanding balance, which is all the account statement and the ageing
/// report need. Both read the same rows so a figure cannot differ between the two screens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacturaPendiente {
    pub id: Uuid,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub numero: String,
    pub fecha: NaiveDate,
    pub fecha_vencimiento: Option<NaiveDate>,
    pub estado: EstadoFactura,
    pub total: Money,
    pub pagado: Money,
}

impl FacturaPendiente {
    /// What is still owed. Never negative: an overpayment is not a credit the ageing can offset.
    pub fn saldo(&self) -> AppResult<Money> {
        let saldo = self.total.checked_sub(self.pagado)?;
        Ok(if saldo.is_negative() {
            Money::ZERO
        } else {
            saldo
        })
    }

    /// The date the arrears are counted from: the due date when it was loaded, and otherwise the
    /// issue date plus the default term.
    ///
    /// Doc 06 §4.5 counts from the issue date, but `Factura::vencimiento_efectivo` already
    /// established the grace period for the invoice list, and the days of arrears the statement
    /// prints have to be the days the list prints.
    pub fn fecha_base(&self, dias_default: u32) -> NaiveDate {
        self.fecha_vencimiento
            .unwrap_or_else(|| self.fecha + chrono::Duration::days(i64::from(dias_default)))
    }
}

/// What the dashboard prints about the installation itself (doc 06 §9.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EstadoBase {
    pub healthy: bool,
    pub migraciones: u64,
    pub tamano_bytes: i64,
}

/// The aggregated reads. Read-only and never paged: every method returns either a scalar or a
/// bounded ranking, and the arithmetic happens in SQL so the dashboard is one round trip per
/// figure rather than one per row.
#[async_trait]
pub trait DashboardRepository: Send + Sync {
    /// Income, expenses, balance and count of the movements booked in the window.
    async fn resumen_rango(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
    ) -> AppResult<MovimientoResumen>;

    /// Customers with at least one **income** movement in the window.
    async fn clientes_activos(&self, desde: DateTime<Utc>, hasta: DateTime<Utc>) -> AppResult<u64>;

    /// Jobs whose state is neither `Finalizado` nor `Cancelado`.
    async fn trabajos_pendientes(&self) -> AppResult<u64>;

    async fn proyectos_pausadas(&self) -> AppResult<u64>;

    /// Invoices past due: explicitly `Vencida`, or issued on or before `umbral`. Both arms also
    /// require an outstanding balance, so a paid invoice never shows up as overdue (doc 06 §9.3).
    async fn facturas_vencidas(&self, umbral: NaiveDate) -> AppResult<u64>;

    /// Active employees with no settlement whose period ends in the given calendar month. This
    /// one KPI is deliberately a calendar month while the rest of the dashboard is a rolling
    /// window (doc 06 §9.4).
    async fn liquidaciones_pendientes(&self, anio: i32, mes: u32) -> AppResult<u64>;

    async fn top_clientes(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
        limite: u64,
    ) -> AppResult<Vec<TotalPorNombre>>;

    async fn gastos_por_categoria(
        &self,
        desde: DateTime<Utc>,
        hasta: DateTime<Utc>,
        limite: u64,
    ) -> AppResult<Vec<TotalPorNombre>>;

    /// The twelve months of `anio`, months without movements included as zero.
    async fn serie_mensual(&self, anio: i32) -> AppResult<Vec<TotalMensual>>;

    /// Sites ranked by profitability. `dir` picks the best or the worst ones.
    async fn rentabilidad_proyectos(
        &self,
        dir: SortDir,
        limite: u64,
    ) -> AppResult<Vec<RentabilidadFila>>;

    /// Jobs ranked by profitability, optionally restricted to one site.
    async fn rentabilidad_trabajos(
        &self,
        proyecto_id: Option<Uuid>,
        limite: u64,
    ) -> AppResult<Vec<RentabilidadFila>>;

    /// Every invoice that counts as debt, for one customer or for all of them. `incluir_pagadas`
    /// widens the read to the settled ones, which the account statement offers as an option.
    async fn facturas_pendientes(
        &self,
        cliente_id: Option<Uuid>,
        incluir_pagadas: bool,
    ) -> AppResult<Vec<FacturaPendiente>>;

    async fn estado_base(&self) -> AppResult<EstadoBase>;
}

