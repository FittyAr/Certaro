//! Repository ports. See `docs/02-arquitectura.md` §5.
//!
//! Use cases only ever see these traits, so the domain can be exercised without a database and a
//! change of ORM stays inside the infrastructure crate.
//!
//! Every read filters `is_deleted = 0` unless the method name says otherwise; there is no flag for
//! it, because an optional flag is a flag someone eventually forgets to pass.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use eo_domain::entities::TipoMovimiento;
use eo_domain::RowVersion;
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

    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}
