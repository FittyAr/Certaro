//! Repository ports. See `docs/02-arquitectura.md` §5.
//!
//! Use cases only ever see these traits, so the domain can be exercised without a database and a
//! change of ORM stays inside the infrastructure crate.
//!
//! Every read filters `is_deleted = 0` unless the method name says otherwise; there is no flag for
//! it, because an optional flag is a flag someone eventually forgets to pass.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use eo_domain::entities::{
    Categoria, Certificado, CertificadoItem, Cliente, ClienteContacto, Factura, Movimiento, Obra,
    OrdenTrabajo, OrdenTrabajoItem, PagoFactura, TipoMovimiento, Trabajo,
};
use eo_domain::{Decimal4, EstadoFactura, EstadoObra, EstadoTrabajo, Moneda, Money, RowVersion};
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

/// Filter of `clientes`. See `docs/09-modulos-funcionales.md` §3.3.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClienteFiltro {
    /// Case-insensitive match against name, CUIT and email.
    pub texto: Option<String>,
    pub condicion_iva: Option<String>,
    /// Only customers with a positive outstanding balance.
    pub solo_con_deuda: bool,
}

/// A customer with the two figures the list shows and the delete case needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClienteConResumen {
    pub cliente: Cliente,
    pub obras_count: u64,
    pub facturas_count: u64,
    /// Sum of the outstanding balance of every invoice that counts as debt. Computed in SQL
    /// because the column is sortable and paging on a value computed in Rust would be wrong.
    pub deuda: Money,
}

#[async_trait]
pub trait ClienteRepository: Send + Sync {
    /// Without contacts: the list does not show them and loading them would be N+1.
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Cliente>>;

    /// With its live contacts, which is what the form edits as a single aggregate.
    async fn find_con_contactos(&self, id: Uuid) -> AppResult<Option<Cliente>>;

    async fn search(
        &self,
        filtro: &ClienteFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<ClienteConResumen>>;

    async fn lookup(&self, texto: Option<&str>, limite: u64) -> AppResult<Vec<Cliente>>;

    async fn insert(&self, entity: &Cliente) -> AppResult<()>;
    async fn update(&self, entity: &Cliente, esperado: RowVersion) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;

    async fn insert_contacto(&self, entity: &ClienteContacto) -> AppResult<()>;
    async fn update_contacto(&self, entity: &ClienteContacto) -> AppResult<()>;
    /// Logical delete of the contacts of `cliente` that are not in `conservar`. Used both when the
    /// form drops a row and, with an empty list, when the customer itself is deleted.
    async fn soft_delete_contactos_excepto(
        &self,
        cliente_id: Uuid,
        conservar: &[Uuid],
        at: DateTime<Utc>,
    ) -> AppResult<()>;

    async fn count_obras(&self, id: Uuid) -> AppResult<u64>;
    async fn count_facturas(&self, id: Uuid) -> AppResult<u64>;
    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64>;
}

/// Filter of `obras`. See `docs/09-modulos-funcionales.md` §3.4.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObraFiltro {
    /// Case-insensitive match against name, number, address and locality.
    pub texto: Option<String>,
    pub cliente_id: Option<Uuid>,
    pub estado: Option<EstadoObra>,
    /// Shorthand for `estado in (Activa, Pausada)`.
    pub solo_activas: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObraConResumen {
    pub obra: Obra,
    pub cliente_nombre: String,
    pub trabajos_count: u64,
    /// Income minus expenses of every movement imputed through one of the site's jobs.
    pub rentabilidad: Money,
}

#[async_trait]
pub trait ObraRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Obra>>;
    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<ObraConResumen>>;

    async fn search(
        &self,
        filtro: &ObraFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<ObraConResumen>>;

    async fn lookup(
        &self,
        cliente_id: Option<Uuid>,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<Obra>>;

    /// Whether the number is taken. Deleted sites count: the number stays reserved (INV-06), and
    /// the unique index is not filtered by `is_deleted`.
    async fn numero_ocupado(&self, numero: i32, excluir: Option<Uuid>) -> AppResult<bool>;

    /// `MAX(numero) + 1` over every row, deleted ones included.
    async fn siguiente_numero(&self) -> AppResult<i32>;

    async fn insert(&self, entity: &Obra) -> AppResult<()>;
    async fn update(&self, entity: &Obra, esperado: RowVersion) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;

    async fn count_trabajos(&self, id: Uuid) -> AppResult<u64>;
    /// The site's jobs that are still open, which block finalising it without cascade.
    async fn trabajos_abiertos(&self, id: Uuid) -> AppResult<Vec<Trabajo>>;
}

/// Filter of `trabajos`. See `docs/09-modulos-funcionales.md` §3.5.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrabajoFiltro {
    pub texto: Option<String>,
    pub obra_id: Option<Uuid>,
    /// Resolved through the site, because a job has no customer of its own.
    pub cliente_id: Option<Uuid>,
    pub estado: Option<EstadoTrabajo>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrabajoConRelaciones {
    pub trabajo: Trabajo,
    pub obra_numero: i32,
    pub obra_nombre: String,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
}

#[async_trait]
pub trait TrabajoRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Trabajo>>;
    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<TrabajoConRelaciones>>;

    async fn search(
        &self,
        filtro: &TrabajoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<TrabajoConRelaciones>>;

    async fn lookup(
        &self,
        obra_id: Option<Uuid>,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<Trabajo>>;

    async fn insert(&self, entity: &Trabajo) -> AppResult<()>;
    async fn update(&self, entity: &Trabajo, esperado: RowVersion) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;

    async fn count_ordenes(&self, id: Uuid) -> AppResult<u64>;
    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64>;
}

/// Filter of `facturas`. See `docs/09-modulos-funcionales.md` §3.8.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FacturaFiltro {
    /// Case-insensitive match against the number and the customer name.
    pub texto: Option<String>,
    pub cliente_id: Option<Uuid>,
    /// Empty means every state; the screen offers a multi-select.
    pub estados: Vec<EstadoFactura>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
    pub solo_impagas: bool,
    pub solo_vencidas: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacturaConResumen {
    pub factura: Factura,
    pub cliente_nombre: String,
    pub pagado: Money,
    pub saldo: Money,
}

#[async_trait]
pub trait FacturaRepository: Send + Sync {
    /// Without payments.
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Factura>>;
    /// With its live payments, which is the aggregate every state recalculation needs.
    async fn find_con_pagos(&self, id: Uuid) -> AppResult<Option<Factura>>;
    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<FacturaConResumen>>;

    async fn search(
        &self,
        filtro: &FacturaFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
        hoy: NaiveDate,
        dias_vencimiento_default: u32,
    ) -> AppResult<PagedResult<FacturaConResumen>>;

    async fn lookup(
        &self,
        cliente_id: Option<Uuid>,
        solo_impagas: bool,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<Factura>>;

    /// Every invoice of a customer that counts as debt, with its payments loaded. Feeds the
    /// account statement and the ageing report.
    async fn de_cliente_con_pagos(
        &self,
        cliente_id: Uuid,
        incluir_pagadas: bool,
    ) -> AppResult<Vec<Factura>>;

    async fn insert(&self, entity: &Factura) -> AppResult<()>;
    async fn update(&self, entity: &Factura, esperado: RowVersion) -> AppResult<()>;
    /// Writes only the state, for the automatic recalculation that follows a payment. It carries
    /// no row version: the caller already holds the row inside the same transaction, and bumping
    /// the version would make the user's next save fail for no reason.
    async fn update_estado(&self, id: Uuid, estado: EstadoFactura) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;

    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64>;

    async fn find_pago(&self, id: Uuid) -> AppResult<Option<PagoFactura>>;
    async fn pagos_de(&self, factura_id: Uuid) -> AppResult<Vec<PagoFactura>>;
    async fn insert_pago(&self, entity: &PagoFactura) -> AppResult<()>;
    async fn update_pago(&self, entity: &PagoFactura, esperado: RowVersion) -> AppResult<()>;
    async fn soft_delete_pago(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()>;
}

/// A work order with the names its screen shows and the figures its list needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdenTrabajoConRelaciones {
    /// With its live items: an order has tens of them, not thousands, and every screen that shows
    /// an order shows its sheet.
    pub orden: OrdenTrabajo,
    pub trabajo_descripcion: String,
    pub obra_id: Uuid,
    pub obra_numero: i32,
    pub obra_nombre: String,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    pub certificados_count: u64,
}

#[async_trait]
pub trait OrdenTrabajoRepository: Send + Sync {
    /// Without items.
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<OrdenTrabajo>>;
    /// With its live items, ordered by `orden`. This is the aggregate the form edits.
    async fn find_con_items(&self, id: Uuid) -> AppResult<Option<OrdenTrabajo>>;
    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<OrdenTrabajoConRelaciones>>;

    /// Every order of a job, with items. Not paged: the doc is explicit that this list is short.
    async fn de_trabajo(&self, trabajo_id: Uuid) -> AppResult<Vec<OrdenTrabajoConRelaciones>>;

    async fn lookup(
        &self,
        trabajo_id: Option<Uuid>,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<OrdenTrabajo>>;

    async fn insert(&self, entity: &OrdenTrabajo) -> AppResult<()>;
    async fn update(&self, entity: &OrdenTrabajo, esperado: RowVersion) -> AppResult<()>;
    /// Bumps only the version and the timestamp of the aggregate root, for the issuing use case:
    /// the order's own fields do not change but the aggregate did (doc 06 §5.5).
    async fn touch(&self, id: Uuid, at: DateTime<Utc>) -> AppResult<()>;
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;

    async fn insert_item(&self, entity: &OrdenTrabajoItem) -> AppResult<()>;
    /// No row version: an item belongs to the order, whose version the caller already checked.
    async fn update_item(&self, entity: &OrdenTrabajoItem) -> AppResult<()>;
    /// Writes just the progress columns, for issuing and voiding a certificate.
    async fn update_avance_item(
        &self,
        id: Uuid,
        porcentaje_anterior: Decimal4,
        porcentaje_actual: Decimal4,
        ejecutado: bool,
        at: DateTime<Utc>,
    ) -> AppResult<()>;
    async fn soft_delete_items_excepto(
        &self,
        orden_trabajo_id: Uuid,
        conservar: &[Uuid],
        at: DateTime<Utc>,
    ) -> AppResult<()>;

    /// The items of the order that already appear in some certificate. They cannot be dropped by
    /// the form: doing so would leave a certified line with nothing to point at.
    async fn items_certificados(&self, orden_trabajo_id: Uuid) -> AppResult<Vec<Uuid>>;
    async fn count_certificados(&self, orden_trabajo_id: Uuid) -> AppResult<u64>;
}

/// Filter of `certificados`. See `docs/09-modulos-funcionales.md` §3.7.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CertificadoFiltro {
    pub obra_id: Option<Uuid>,
    pub trabajo_id: Option<Uuid>,
    pub cliente_id: Option<Uuid>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificadoConRelaciones {
    pub certificado: Certificado,
    pub orden_trabajo_id: Uuid,
    pub orden_titulo: String,
    pub trabajo_id: Uuid,
    pub trabajo_descripcion: String,
    pub obra_id: Uuid,
    pub obra_numero: i32,
    pub obra_nombre: String,
    pub cliente_id: Uuid,
    pub cliente_nombre: String,
    /// Whether this is the highest-numbered certificate of its order, which is the only one that
    /// can be voided (doc 06 §5.6).
    pub es_ultimo: bool,
}

#[async_trait]
pub trait CertificadoRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Certificado>>;
    async fn find_con_items(&self, id: Uuid) -> AppResult<Option<Certificado>>;
    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<CertificadoConRelaciones>>;

    async fn search(
        &self,
        filtro: &CertificadoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<CertificadoConRelaciones>>;

    /// The certificates of one order, newest number first.
    async fn de_orden(&self, orden_trabajo_id: Uuid) -> AppResult<Vec<Certificado>>;

    /// `MAX(numero)` of the order, deleted rows included: a number is never reused (INV-15).
    async fn ultimo_numero(&self, orden_trabajo_id: Uuid) -> AppResult<i32>;

    /// What each item of the order has certified so far, as `(orden_trabajo_item_id, porcentaje)`.
    /// Read from the certificates rather than from the item so the check does not depend on the
    /// column the use case is about to write.
    async fn acumulado_por_item(&self, orden_trabajo_id: Uuid) -> AppResult<Vec<(Uuid, Decimal4)>>;

    async fn insert(&self, entity: &Certificado) -> AppResult<()>;
    async fn insert_item(&self, entity: &CertificadoItem) -> AppResult<()>;
    /// Writes only the notes: everything else in an issued certificate is immutable.
    async fn update_observaciones(
        &self,
        id: Uuid,
        observaciones: Option<&str>,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()>;
    /// Deletes the certificate and its lines. The number stays spent.
    async fn soft_delete(&self, id: Uuid, esperado: RowVersion, at: DateTime<Utc>)
        -> AppResult<()>;
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
    fn clientes(&self) -> &dyn ClienteRepository;
    fn obras(&self) -> &dyn ObraRepository;
    fn trabajos(&self) -> &dyn TrabajoRepository;
    fn facturas(&self) -> &dyn FacturaRepository;
    fn ordenes_trabajo(&self) -> &dyn OrdenTrabajoRepository;
    fn certificados(&self) -> &dyn CertificadoRepository;

    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}
