//! Repository ports. See `docs/02-arquitectura.md` §5.
//!
//! Use cases only ever see these traits, so the domain can be exercised without a database and a
//! change of ORM stays inside the infrastructure crate.
//!
//! Every read filters `is_deleted = 0` unless the method name says otherwise; there is no flag for
//! it, because an optional flag is a flag someone eventually forgets to pass.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use eo_domain::entities::{Categoria, Movimiento, TipoMovimiento};
use eo_domain::{Moneda, Money, RowVersion};
use uuid::Uuid;

use crate::paging::{PageRequest, PagedResult};
use crate::result::AppResult;

/// How a list is sorted. The set of accepted `field` values is validated per module, because an
/// arbitrary column name coming from the frontend would be an injection vector.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

/// Filter of `tipos_movimiento`. A typed struct rather than free SQL: the application layer
/// describes what it wants, and only the repository knows how that becomes a query.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TipoMovimientoFiltro {
    /// Case-insensitive match against the name and the description.
    pub texto: Option<String>,
    pub es_ingreso: Option<bool>,
    pub es_sistema: Option<bool>,
}

/// A name and how many movements use it, which is what the list screen shows and what the delete
/// case needs in order to refuse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TipoMovimientoConUso {
    pub tipo: TipoMovimiento,
    pub movimientos_count: u64,
}

#[async_trait]
pub trait TipoMovimientoRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<TipoMovimiento>>;

    /// Looks a name up for the uniqueness check, ignoring case and ignoring `excluir` so that
    /// renaming a row to its own name is not a conflict.
    async fn find_by_nombre(
        &self,
        nombre: &str,
        excluir: Option<Uuid>,
    ) -> AppResult<Option<TipoMovimiento>>;

    async fn search(
        &self,
        filtro: &TipoMovimientoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<TipoMovimientoConUso>>;

    /// Options for a selector: no counts, no pagination metadata, at most `limite` rows.
    async fn lookup(&self, texto: Option<&str>, limite: u64) -> AppResult<Vec<TipoMovimiento>>;

    async fn insert(&self, entity: &TipoMovimiento) -> AppResult<()>;

    /// Updates the row whose version still matches; a mismatch is reported as a concurrency
    /// error rather than silently overwriting somebody else's edit.
    async fn update(&self, entity: &TipoMovimiento, esperado: RowVersion) -> AppResult<()>;

    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;

    /// How many live movements point at this type. Zero is the condition for deleting it.
    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64>;
}

/// Filter of `categorias`. See `docs/09-modulos-funcionales.md` §3.13.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CategoriaFiltro {
    pub texto: Option<String>,
    /// `Some(None)` asks for the root ones; `None` does not filter by parent at all.
    pub categoria_padre_id: Option<Option<Uuid>>,
}

/// A category with what the list screen needs to decide whether it can be deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoriaConUso {
    pub categoria: Categoria,
    pub movimientos_count: u64,
    pub hijas_count: u64,
    /// Name of the parent, resolved so the list does not issue one query per row.
    pub padre_nombre: Option<String>,
}

#[async_trait]
pub trait CategoriaRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Categoria>>;

    /// Uniqueness is per parent: two sibling categories cannot share a name, but `Materiales`
    /// under two different parents is fine.
    async fn find_by_nombre(
        &self,
        nombre: &str,
        padre: Option<Uuid>,
        excluir: Option<Uuid>,
    ) -> AppResult<Option<Categoria>>;

    async fn search(
        &self,
        filtro: &CategoriaFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<CategoriaConUso>>;

    async fn lookup(&self, texto: Option<&str>, limite: u64) -> AppResult<Vec<Categoria>>;

    async fn insert(&self, entity: &Categoria) -> AppResult<()>;
    async fn update(&self, entity: &Categoria, esperado: RowVersion) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;

    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64>;
    async fn count_hijas(&self, id: Uuid) -> AppResult<u64>;

    /// The chain of ancestors of `id`, closest first. Used to reject a cycle deeper than one
    /// level, which the field validation cannot see.
    async fn ancestros(&self, id: Uuid) -> AppResult<Vec<Uuid>>;
}

/// Filter of `movimientos`. See `docs/06-casos-de-uso-y-formulas.md` §3.3.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MovimientoFiltro {
    /// Case-insensitive substring of the concept.
    pub concepto: Option<String>,
    pub tipo_movimiento_id: Option<Uuid>,
    pub categoria_id: Option<Uuid>,
    pub cliente_id: Option<Uuid>,
    pub trabajo_id: Option<Uuid>,
    pub empleado_id: Option<Uuid>,
    pub factura_id: Option<Uuid>,
    pub moneda: Option<Moneda>,
    /// Civil dates; the repository widens them to cover the whole day in UTC.
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
    /// Compared against `monto`, not against the total.
    pub monto_min: Option<Money>,
    pub monto_max: Option<Money>,
}

/// A movement plus the names the listing shows, resolved in the same query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovimientoConRelaciones {
    pub movimiento: Movimiento,
    pub tipo_movimiento_nombre: String,
    pub es_ingreso: bool,
    pub categoria_nombre: Option<String>,
    pub categoria_color: Option<String>,
    /// True when an advance was already consumed by a payroll run and so cannot be edited.
    pub bloqueado_por_liquidacion: bool,
}

/// Totals over the **whole filter**, not over the page the user is looking at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovimientoResumen {
    pub total_ingresos: Money,
    pub total_gastos: Money,
    pub balance: Money,
    pub cantidad: u64,
}

#[async_trait]
pub trait MovimientoRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Movimiento>>;

    /// The same row as `find_by_id` with the names the detail screen shows.
    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<MovimientoConRelaciones>>;

    async fn search(
        &self,
        filtro: &MovimientoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<MovimientoConRelaciones>>;

    /// Income, expenses and balance for the filter. Separate from `search` because the summary
    /// covers every matching row while the page covers thirty of them.
    async fn resumen(&self, filtro: &MovimientoFiltro) -> AppResult<MovimientoResumen>;

    async fn insert(&self, entity: &Movimiento) -> AppResult<()>;
    async fn update(&self, entity: &Movimiento, esperado: RowVersion) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;

    /// Whether the movement is referenced by a live payroll advance. A `RESTRICT` foreign key
    /// backs this, but asking first turns a database error into a message the user understands.
    async fn esta_en_liquidacion(&self, id: Uuid) -> AppResult<bool>;

    /// Existence check for a foreign key, without loading the row.
    async fn existe_referencia(&self, tabla: ReferenciaTabla, id: Uuid) -> AppResult<bool>;
}

/// The tables a movement can point at. A closed list rather than a table name as a string,
/// because that string would end up interpolated into SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenciaTabla {
    TipoMovimiento,
    TipoConceptoPago,
    Categoria,
    Cliente,
    Trabajo,
    Empleado,
    Factura,
}

impl ReferenciaTabla {
    /// Field of the input the check belongs to, for the resulting error.
    pub const fn campo(self) -> &'static str {
        match self {
            Self::TipoMovimiento => "tipoMovimientoId",
            Self::TipoConceptoPago => "tipoConceptoPagoId",
            Self::Categoria => "categoriaId",
            Self::Cliente => "clienteId",
            Self::Trabajo => "trabajoId",
            Self::Empleado => "empleadoId",
            Self::Factura => "facturaId",
        }
    }

    pub const fn entidad(self) -> &'static str {
        match self {
            Self::TipoMovimiento => "TipoMovimiento",
            Self::TipoConceptoPago => "TipoConceptoPago",
            Self::Categoria => "Categoria",
            Self::Cliente => "Cliente",
            Self::Trabajo => "Trabajo",
            Self::Empleado => "Empleado",
            Self::Factura => "Factura",
        }
    }
}

/// Opens transactions. A use case that writes more than one table has to go through this.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn begin(&self) -> AppResult<Box<dyn Transaction>>;
}

/// A transaction with one accessor per aggregate. Dropping it without committing rolls back.
///
/// `Sync` as well as `Send` because a use case holds `&dyn Transaction` across an `await`, and a
/// future that does that is only `Send` if the reference is.
#[async_trait]
pub trait Transaction: Send + Sync {
    fn tipos_movimiento(&self) -> &dyn TipoMovimientoRepository;
    fn categorias(&self) -> &dyn CategoriaRepository;
    fn movimientos(&self) -> &dyn MovimientoRepository;

    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}
