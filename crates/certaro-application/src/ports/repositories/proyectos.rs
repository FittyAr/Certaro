use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;
use certaro_domain::entities::*;
use certaro_domain::{Decimal4, EstadoFactura, EstadoProyecto, EstadoTrabajo, Moneda, Money, RowVersion};
use crate::paging::{PageRequest, PagedResult};
use crate::result::AppResult;
use super::common::*;

/// Filter of `proyectos`. See `docs/09-modulos-funcionales.md` §3.4.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProyectoFiltro {
    /// Case-insensitive match against name, number, address and locality.
    pub texto: Option<String>,
    pub cliente_id: Option<Uuid>,
    pub estado: Option<EstadoProyecto>,
    /// Shorthand for `estado in (Activa, Pausada)`.
    pub solo_activas: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProyectoConResumen {
    pub proyecto: Proyecto,
    pub cliente_nombre: String,
    pub trabajos_count: u64,
    /// Income minus expenses of every movement imputed through one of the site's jobs.
    pub rentabilidad: Money,
}

#[async_trait]
pub trait ProyectoRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Proyecto>>;
    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<ProyectoConResumen>>;

    async fn search(
        &self,
        filtro: &ProyectoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<ProyectoConResumen>>;

    async fn lookup(
        &self,
        cliente_id: Option<Uuid>,
        texto: Option<&str>,
        limite: u64,
    ) -> AppResult<Vec<Proyecto>>;

    /// Whether the number is taken. Deleted sites count: the number stays reserved (INV-06), and
    /// the unique index is not filtered by `is_deleted`.
    async fn numero_ocupado(&self, numero: i32, excluir: Option<Uuid>) -> AppResult<bool>;

    /// `MAX(numero) + 1` over every row, deleted ones included.
    async fn siguiente_numero(&self) -> AppResult<i32>;

    async fn insert(&self, entity: &Proyecto) -> AppResult<()>;
    async fn update(&self, entity: &Proyecto, esperado: RowVersion) -> AppResult<()>;
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
    pub proyecto_id: Option<Uuid>,
    /// Resolved through the site, because a job has no customer of its own.
    pub cliente_id: Option<Uuid>,
    pub estado: Option<EstadoTrabajo>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrabajoConRelaciones {
    pub trabajo: Trabajo,
    pub proyecto_numero: i32,
    pub proyecto_nombre: String,
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
        proyecto_id: Option<Uuid>,
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

/// A work order with the names its screen shows and the figures its list needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdenTrabajoConRelaciones {
    /// With its live items: an order has tens of them, not thousands, and every screen that shows
    /// an order shows its sheet.
    pub orden: OrdenTrabajo,
    pub trabajo_descripcion: String,
    pub proyecto_id: Uuid,
    pub proyecto_numero: i32,
    pub proyecto_nombre: String,
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

    /// List orders, optionally filtered by job. When trabajo_id is None, returns all active orders.
    async fn listar(&self, trabajo_id: Option<Uuid>) -> AppResult<Vec<OrdenTrabajoConRelaciones>>;

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
    pub orden_trabajo_id: Option<Uuid>,
    pub proyecto_id: Option<Uuid>,
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
    pub proyecto_id: Uuid,
    pub proyecto_numero: i32,
    pub proyecto_nombre: String,
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
