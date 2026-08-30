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
    AsistenciaEmpleado, Categoria, Certificado, CertificadoItem, Cliente, ClienteContacto,
    Empleado, Factura, Feriado, Liquidacion, LiquidacionAdelanto, Movimiento, Obra, OrdenTrabajo,
    OrdenTrabajoItem, PagoFactura, TipoMovimiento, Trabajo,
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
    pub cliente_nombre: Option<String>,
    pub trabajo_descripcion: Option<String>,
    /// The site the job belongs to. Resolved through the job, which is the only path a movement
    /// has to a site, and needed by the export columns (doc 12 §2.2).
    pub obra_nombre: Option<String>,
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

#[async_trait]
pub trait FeriadoRepository: Send + Sync {
    /// The holidays of a closed range of civil dates. The settlement reads the table, never the
    /// network.
    async fn del_rango(&self, desde: NaiveDate, hasta: NaiveDate) -> AppResult<Vec<Feriado>>;
    async fn del_anio(&self, anio: i32) -> AppResult<Vec<Feriado>>;
    async fn count_anio(&self, anio: i32) -> AppResult<u64>;

    /// Inserts what is missing and leaves every existing row alone, so a hand-added holiday is
    /// never overwritten by a sync. Returns how many rows were added.
    async fn insertar_faltantes(&self, feriados: &[Feriado]) -> AppResult<u64>;

    /// Upsert of a hand-added holiday.
    async fn upsert_manual(&self, entity: &Feriado) -> AppResult<()>;

    /// A real delete: a holiday left behind would keep paying its multiplier.
    async fn delete(&self, fecha: NaiveDate) -> AppResult<()>;
}

/// A total with the name it is grouped by, for the top-customers and top-categories rankings.
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

    async fn obras_pausadas(&self) -> AppResult<u64>;

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
    async fn rentabilidad_obras(
        &self,
        dir: SortDir,
        limite: u64,
    ) -> AppResult<Vec<RentabilidadFila>>;

    /// Jobs ranked by profitability, optionally restricted to one site.
    async fn rentabilidad_trabajos(
        &self,
        obra_id: Option<Uuid>,
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

/// The internal key/value store. Not a business record: no audit block, no soft delete.
#[async_trait]
pub trait MetadataRepository: Send + Sync {
    /// The value and when it was written, which is what a cache needs to know if it expired.
    async fn get(&self, key: &str) -> AppResult<Option<(String, DateTime<Utc>)>>;
    async fn set(&self, key: &str, value: &str, at: DateTime<Utc>) -> AppResult<()>;
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
    fn empleados(&self) -> &dyn EmpleadoRepository;
    fn asistencias(&self) -> &dyn AsistenciaRepository;
    fn liquidaciones(&self) -> &dyn LiquidacionRepository;
    fn feriados(&self) -> &dyn FeriadoRepository;
    fn dashboard(&self) -> &dyn DashboardRepository;
    fn metadata(&self) -> &dyn MetadataRepository;

    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}
