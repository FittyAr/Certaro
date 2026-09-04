use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;
use certaro_domain::entities::*;
use certaro_domain::{Decimal4, EstadoFactura, EstadoProyecto, EstadoTrabajo, Moneda, Money, RowVersion};
use crate::paging::{PageRequest, PagedResult};
use crate::result::AppResult;
use super::common::*;

/// Filter of `empleados`. See `docs/09-modulos-funcionales.md` §3.9.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmpleadoFiltro {
    /// Case-insensitive match against name, document and role.
    pub texto: Option<String>,
    /// `None` means every employee; the list screen defaults to `Some(true)`.
    pub activo: Option<bool>,
    pub cargo: Option<String>,
}

#[async_trait]
pub trait EmpleadoRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Empleado>>;

    async fn search(
        &self,
        filtro: &EmpleadoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<Empleado>>;

    async fn lookup(
        &self,
        texto: Option<&str>,
        solo_activos: bool,
        limite: u64,
    ) -> AppResult<Vec<Empleado>>;

    /// Every active employee, by name: the attendance grid and the settlement wizard both list
    /// them in full rather than paged.
    async fn activos(&self) -> AppResult<Vec<Empleado>>;

    /// The distinct roles in use, for the filter dropdown.
    async fn cargos(&self) -> AppResult<Vec<String>>;

    async fn insert(&self, entity: &Empleado) -> AppResult<()>;
    async fn update(&self, entity: &Empleado, esperado: RowVersion) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;

    async fn count_liquidaciones(&self, id: Uuid) -> AppResult<u64>;
    async fn count_asistencias(&self, id: Uuid) -> AppResult<u64>;
    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64>;
}

#[async_trait]
pub trait AsistenciaRepository: Send + Sync {
    /// The record of one employee on one day, which is the natural key the grid writes by.
    ///
    /// Deleted rows included, unlike every other read: the unique index covers them, so a cleared
    /// cell has to be revived rather than inserted again.
    async fn find_por_empleado_fecha(
        &self,
        empleado_id: Uuid,
        fecha: NaiveDate,
    ) -> AppResult<Option<AsistenciaEmpleado>>;

    /// Every record of the period, for the employees given or for all of them when the list is
    /// empty. One query feeds the whole grid.
    async fn del_periodo(
        &self,
        desde: NaiveDate,
        hasta: NaiveDate,
        empleados: &[Uuid],
    ) -> AppResult<Vec<AsistenciaEmpleado>>;

    async fn insert(&self, entity: &AsistenciaEmpleado) -> AppResult<()>;
    async fn update(&self, entity: &AsistenciaEmpleado) -> AppResult<()>;

    /// Clears a cell. Takes the natural key because that is what the grid knows.
    async fn soft_delete_por_empleado_fecha(
        &self,
        empleado_id: Uuid,
        fecha: NaiveDate,
        at: DateTime<Utc>,
    ) -> AppResult<()>;
}

/// Filter of `liquidaciones`. See `docs/09-modulos-funcionales.md` §3.11.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LiquidacionFiltro {
    pub empleado_id: Option<Uuid>,
    /// Matched against the period, not against `created_at`.
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
    pub solo_sin_pdf: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiquidacionConRelaciones {
    pub liquidacion: Liquidacion,
    pub empleado_nombre: String,
    pub empleado_cargo: Option<String>,
    pub empleado_dni: Option<String>,
}

/// An advance that a settlement could take, with whether it is already spent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdelantoCandidato {
    pub movimiento_id: Uuid,
    pub fecha: NaiveDate,
    pub concepto: String,
    /// `monto × cantidad`: the total of the movement, not its unit price.
    pub monto: Money,
    /// The settlement that already consumed it, if any (INV-05).
    pub liquidacion_id: Option<Uuid>,
}

#[async_trait]
pub trait LiquidacionRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Liquidacion>>;
    async fn find_con_adelantos(&self, id: Uuid) -> AppResult<Option<Liquidacion>>;
    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<LiquidacionConRelaciones>>;

    async fn search(
        &self,
        filtro: &LiquidacionFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<LiquidacionConRelaciones>>;

    /// Whether the employee already has a live settlement overlapping the period (doc 06 §6.8).
    async fn periodo_solapado(
        &self,
        empleado_id: Uuid,
        desde: NaiveDate,
        hasta: NaiveDate,
        excluir: Option<Uuid>,
    ) -> AppResult<Option<Liquidacion>>;

    /// The advances of the employee inside the period, each flagged with the settlement that
    /// already consumed it. Reading them instead of only summing is what lets the wizard show and
    /// strike out a spent advance.
    async fn adelantos_candidatos(
        &self,
        empleado_id: Uuid,
        desde: NaiveDate,
        hasta: NaiveDate,
    ) -> AppResult<Vec<AdelantoCandidato>>;

    async fn insert(&self, entity: &Liquidacion) -> AppResult<()>;
    async fn update(&self, entity: &Liquidacion, esperado: RowVersion) -> AppResult<()>;
    async fn insert_adelanto(&self, entity: &LiquidacionAdelanto) -> AppResult<()>;
    async fn adelantos_de(&self, liquidacion_id: Uuid) -> AppResult<Vec<LiquidacionAdelanto>>;
    async fn marcar_pdf_generado(&self, id: Uuid, at: DateTime<Utc>) -> AppResult<()>;

    /// Deletes the settlement and its advances, which frees them to be settled again.
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;
}

