//! Contract of the `dashboard` module. See `docs/11-contratos-tauri.md` §5.10 and
//! `docs/06-casos-de-uso-y-formulas.md` §9.

use chrono::{DateTime, Utc};
use certaro_domain::{Decimal4, Money};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::DashboardPeriod;
use crate::dtos::movimientos::MovimientoListItem;
use crate::ports::repositories::{RentabilidadFila, TotalMensual, TotalPorNombre};

/// The window the dashboard aggregates over.
///
/// `Total` is the third period doc 06 §9.1 calls `Historico`; the contract in doc 11 §5.10 and the
/// configuration both say `Total`, and two out of three wins.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PeriodoDashboard {
    #[default]
    Mensual,
    Anual,
    Total,
}

impl From<DashboardPeriod> for PeriodoDashboard {
    fn from(p: DashboardPeriod) -> Self {
        match p {
            DashboardPeriod::Mensual => Self::Mensual,
            DashboardPeriod::Anual => Self::Anual,
            DashboardPeriod::Total => Self::Total,
        }
    }
}

impl From<PeriodoDashboard> for DashboardPeriod {
    fn from(p: PeriodoDashboard) -> Self {
        match p {
            PeriodoDashboard::Mensual => Self::Mensual,
            PeriodoDashboard::Anual => Self::Anual,
            PeriodoDashboard::Total => Self::Total,
        }
    }
}

/// One month of the yearly series. Both signs travel together because the chart draws them as two
/// lines over one axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PuntoSerie {
    /// 1 to 12. The label is built in the frontend, which owns the locale.
    pub mes: u32,
    pub ingresos: Money,
    pub gastos: Money,
}

impl From<TotalMensual> for PuntoSerie {
    fn from(row: TotalMensual) -> Self {
        Self {
            mes: row.mes,
            ingresos: row.ingresos,
            gastos: row.gastos,
        }
    }
}

/// A ranking row. Used for both top customers and expenses by category: the shape is the same and
/// the screen labels it differently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopCliente {
    /// Absent for a grouping that is not navigable.
    pub id: Option<Uuid>,
    pub nombre: String,
    pub total: Money,
}

impl From<TotalPorNombre> for TopCliente {
    fn from(row: TotalPorNombre) -> Self {
        Self {
            id: row.id,
            nombre: row.nombre,
            total: row.total,
        }
    }
}

/// Profitability of a site or a job, with the margin already computed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RentabilidadItem {
    pub id: Uuid,
    pub nombre: String,
    /// Site the job belongs to. Empty when the row is a site.
    pub contexto: String,
    pub ingresos: Money,
    pub gastos: Money,
    pub rentabilidad: Money,
    /// Zero when there is no income, never `null` and never a division by zero (doc 06 §7.1).
    pub margen_porcentaje: Decimal4,
}

impl RentabilidadItem {
    /// `(rentabilidad / ingresos) × 100`, rounded to two decimals, or zero when there is nothing
    /// to divide by. The order of operations is the documented one: divide, scale, then round.
    pub fn margen(ingresos: Money, rentabilidad: Money) -> Decimal4 {
        if !ingresos.is_positive() {
            return Decimal4::ZERO;
        }
        Decimal4::from_raw(rentabilidad.raw())
            .checked_div(Decimal4::from_raw(ingresos.raw()))
            .and_then(|q| q.checked_mul(Decimal4::HUNDRED))
            .map(|p| p.round_to(2))
            .unwrap_or(Decimal4::ZERO)
    }
}

impl From<RentabilidadFila> for RentabilidadItem {
    fn from(row: RentabilidadFila) -> Self {
        Self {
            id: row.id,
            nombre: row.etiqueta,
            contexto: row.contexto,
            ingresos: row.ingresos,
            gastos: row.gastos,
            rentabilidad: row.rentabilidad,
            margen_porcentaje: Self::margen(row.ingresos, row.rentabilidad),
        }
    }
}

/// What the dashboard prints about the installation. `estado` is an i18n key, not a sentence
/// (doc 06 §9.10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoSistema {
    pub version: String,
    pub base_saludable: bool,
    pub estado: String,
    pub migraciones: u64,
    pub tamano_bytes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub periodo: PeriodoDashboard,
    pub desde: DateTime<Utc>,
    pub hasta: DateTime<Utc>,
    pub total_ingresos: Money,
    pub total_gastos: Money,
    /// Derived, never stored.
    pub balance: Money,
    pub cantidad_movimientos: u64,
    /// `(balance / ingresos) × 100`, or zero without income.
    pub rentabilidad: Decimal4,
    pub anterior_ingresos: Money,
    pub anterior_gastos: Money,
    /// `null` means "no basis for comparison": the previous period was zero and the current one is
    /// not, so the screen shows a dash rather than an infinity (doc 06 §9.5).
    pub variacion_ingresos: Option<Decimal4>,
    pub variacion_gastos: Option<Decimal4>,
    pub variacion_balance: Option<Decimal4>,
    pub clientes_activos: u64,
    pub trabajos_pendientes: u64,
    pub proyectos_pausadas: u64,
    pub facturas_vencidas: u64,
    pub liquidaciones_pendientes: u64,
    /// The calendar year in progress, not the selected period (doc 06 §9.8).
    pub serie_mensual: Vec<PuntoSerie>,
    pub top_clientes: Vec<TopCliente>,
    pub gastos_por_categoria: Vec<TopCliente>,
    pub mejores_proyectos: Vec<RentabilidadItem>,
    pub peores_proyectos: Vec<RentabilidadItem>,
    pub ultimos_movimientos: Vec<MovimientoListItem>,
    pub estado_sistema: EstadoSistema,
}

impl DashboardStats {
    /// `((actual − anterior) / anterior) × 100`, to one decimal.
    ///
    /// Zero over zero is zero, not `null`: nothing changed. Something over zero has no percentage
    /// at all, and saying `+100 %` there would be an invention.
    pub fn variacion(anterior: Money, actual: Money) -> Option<Decimal4> {
        if anterior.is_zero() {
            return if actual.is_zero() {
                Some(Decimal4::ZERO)
            } else {
                None
            };
        }
        actual
            .checked_sub(anterior)
            .ok()
            .map(|delta| Decimal4::from_raw(delta.raw()))
            .and_then(|delta| delta.checked_div(Decimal4::from_raw(anterior.raw())).ok())
            .and_then(|q| q.checked_mul(Decimal4::HUNDRED).ok())
            .map(|p| p.round_to(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum SeveridadAlerta {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum TipoAlerta {
    FacturasVencidas,
    BalanceNegativo,
    ProyectosPausados,
    LiquidacionesPendientes,
    CaidaIngresos,
}

/// An actionable alert. `destino` is the route with its filter already applied, so the card is a
/// link and not just a warning (doc 09 §3.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alerta {
    pub tipo: TipoAlerta,
    /// i18n key of the message.
    pub clave: String,
    pub cantidad: u64,
    /// Set instead of `cantidad` for the alerts whose subject is an amount.
    pub monto: Option<Money>,
    pub severidad: SeveridadAlerta,
    pub destino: String,
}
