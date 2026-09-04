use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;
use certaro_domain::entities::*;
use certaro_domain::{Decimal4, EstadoFactura, EstadoProyecto, EstadoTrabajo, Moneda, Money, RowVersion};
use crate::paging::{PageRequest, PagedResult};
use crate::result::AppResult;
use super::common::*;

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
    pub proyectos_count: u64,
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

    async fn count_proyectos(&self, id: Uuid) -> AppResult<u64>;
    async fn count_facturas(&self, id: Uuid) -> AppResult<u64>;
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
