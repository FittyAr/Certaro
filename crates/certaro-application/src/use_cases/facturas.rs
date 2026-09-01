//! Use cases of `facturas` and its payments. See `docs/09-modulos-funcionales.md` §3.8.
//!
//! Every write that can move the balance ends with a state recalculation inside the same
//! transaction. The legacy system inserted the payment and left the state alone, so a fully paid
//! invoice stayed `Emitida` forever and kept showing up in the debt and in the overdue count; it
//! was the most visible functional defect of the old application.

use std::sync::Arc;

use chrono::NaiveDate;
use certaro_domain::entities::{Audit, Factura, PagoFactura};
use certaro_domain::{recalcular_estado_factura, EstadoFactura, Money, StateMachine};
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::{ListQuery, LookupItem};
use crate::dtos::facturas::{
    FacturaDetalle, FacturaFiltroDto, FacturaInput, FacturaListItem, PagoFacturaInput,
    PagoFacturaItem,
};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{FacturaRepository, Transaction, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort, SettingsStore};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;
use crate::validation::movimientos::ContextoFecha;

const ENTITY: &str = "Factura";
const ENTITY_PAGO: &str = "PagoFactura";

const SORTABLE: [&str; 6] = [
    "numero",
    "fecha",
    "fechaVencimiento",
    "clienteNombre",
    "total",
    "estado",
];

pub struct FacturasService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
    settings: Arc<dyn SettingsStore>,
}

impl FacturasService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
        settings: Arc<dyn SettingsStore>,
    ) -> Self {
        Self {
            uow,
            clock,
            ids,
            settings,
        }
    }

    /// Today as a civil date. Everything overdue-related is measured in days, so the instant is
    /// noise; what matters is which day it is.
    fn hoy(&self) -> NaiveDate {
        self.clock.now_utc().date_naive()
    }

    fn dias_vencimiento(&self) -> u32 {
        self.settings
            .snapshot()
            .business
            .factura_dias_vencimiento_default
    }

    pub async fn list(
        &self,
        query: ListQuery<FacturaFiltroDto>,
    ) -> AppResult<PagedResult<FacturaListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();
        let hoy = self.hoy();
        let dias = self.dias_vencimiento();

        let tx = self.uow.begin().await?;
        let result = tx
            .facturas()
            .search(&filtro, page, sort_by, query.sort_dir, hoy, dias)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(|row| FacturaListItem::build(row, hoy, dias)))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<FacturaDetalle> {
        let hoy = self.hoy();
        let dias = self.dias_vencimiento();

        let tx = self.uow.begin().await?;
        let loaded = load_detalle(&*tx, id, hoy, dias).await;
        finish_read(tx, loaded).await
    }

    pub async fn lookup(
        &self,
        cliente_id: Option<Uuid>,
        solo_impagas: bool,
        texto: Option<String>,
        limite: Option<u64>,
    ) -> AppResult<Vec<LookupItem>> {
        let tx = self.uow.begin().await?;
        let result = tx
            .facturas()
            .lookup(
                cliente_id,
                solo_impagas,
                texto.as_deref(),
                limite.unwrap_or(50),
            )
            .await;
        let facturas = finish_read(tx, result).await?;
        Ok(facturas
            .into_iter()
            .map(|f| {
                LookupItem::new(f.id, f.numero)
                    .with_meta("clienteId", f.cliente_id.to_string())
                    .with_meta("total", f.total.to_decimal_string())
            })
            .collect())
    }

    pub async fn create(&self, input: FacturaInput) -> AppResult<FacturaDetalle> {
        let input = con_total_recalculado(input);
        validation::facturas::validate_factura(&input)?;

        let now = self.clock.now_utc();
        let hoy = self.hoy();
        let dias = self.dias_vencimiento();

        let entity = Factura {
            id: self.ids.new_id(),
            numero: input.numero.trim().to_owned(),
            fecha: input.fecha,
            // Filled in when absent so the arrears report has a date to work with; leaving it null
            // would make the invoice look like it is never due.
            fecha_vencimiento: Some(
                input
                    .fecha_vencimiento
                    .unwrap_or_else(|| input.fecha + chrono::Duration::days(i64::from(dias))),
            ),
            cliente_id: input.cliente_id,
            estado: EstadoFactura::Borrador,
            subtotal: input.subtotal,
            iva: input.iva,
            total: input.total,
            observaciones: normalise(input.observaciones),
            pagos: Vec::new(),
            audit: Audit::new(now),
        };

        let tx = self.uow.begin().await?;
        let outcome = async {
            ensure_cliente_existe(&*tx, entity.cliente_id).await?;
            tx.facturas().insert(&entity).await?;
            load_detalle(&*tx, entity.id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, numero = %detalle.numero, "factura creada");
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: FacturaInput,
        row_version: &str,
    ) -> AppResult<FacturaDetalle> {
        let input = con_total_recalculado(input);
        validation::facturas::validate_factura(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();
        let hoy = self.hoy();
        let dias = self.dias_vencimiento();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.facturas();
            let mut entity = repo
                .find_con_pagos(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            ensure_cliente_existe(&*tx, input.cliente_id).await?;

            entity.numero = input.numero.trim().to_owned();
            entity.fecha = input.fecha;
            entity.fecha_vencimiento = input.fecha_vencimiento.or(entity.fecha_vencimiento);
            entity.cliente_id = input.cliente_id;
            entity.subtotal = input.subtotal;
            entity.iva = input.iva;
            entity.total = input.total;
            entity.observaciones = normalise(input.observaciones);
            entity.audit.touch(now);

            // T-F10: changing the total or the due date can make the invoice paid or overdue.
            recalcular_estado_factura(&mut entity, hoy, dias)?;
            repo.update(&entity, esperado).await?;

            load_detalle(&*tx, id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "factura actualizada");
        Ok(detalle)
    }

    /// T-F02 … T-F06. The automatic states are unreachable from here by construction: they are
    /// absent from `allowed_targets`.
    pub async fn transition(
        &self,
        id: Uuid,
        destino: EstadoFactura,
        row_version: &str,
    ) -> AppResult<FacturaDetalle> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();
        let hoy = self.hoy();
        let dias = self.dias_vencimiento();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.facturas();
            let mut entity = repo
                .find_con_pagos(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let pagos = entity.pagos.iter().filter(|p| !p.audit.is_deleted).count();

            // T-F04, T-F05 and T-F06: annulling or un-issuing an invoice with money against it
            // would orphan the payments. They have to be removed first, deliberately.
            if pagos > 0 && matches!(destino, EstadoFactura::Anulada | EstadoFactura::Borrador) {
                return Err(AppError::Conflict {
                    code: "FACTURA_CON_PAGOS",
                    message_key: "State.Factura.TienePagos",
                    params: [("count".to_owned(), pagos.to_string())].into(),
                });
            }

            // T-F02: issuing an invoice for nothing is a mistake, not a document.
            if destino == EstadoFactura::Emitida && !entity.total.is_positive() {
                return Err(AppError::conflict(
                    "FACTURA_TOTAL_NO_POSITIVO",
                    "State.Factura.RequiereTotalPositivo",
                ));
            }

            entity.estado = entity.estado.transition_to(destino)?;
            // Issuing may land straight on `Vencida` when the invoice is backdated past its due
            // date, which is exactly what happens when someone loads last month's paperwork.
            recalcular_estado_factura(&mut entity, hoy, dias)?;
            entity.audit.touch(now);
            repo.update(&entity, esperado).await?;

            load_detalle(&*tx, id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, estado = %detalle.estado.actual, "factura transicionada");
        Ok(detalle)
    }

    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.facturas();
            let factura = repo
                .find_con_pagos(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let pagos = factura.pagos.iter().filter(|p| !p.audit.is_deleted).count();
            if pagos > 0 {
                return Err(AppError::DependencyInUse {
                    code: "FACTURA_CON_PAGOS",
                    message_key: "Conflict.Factura.ConPagos",
                    params: [("count".to_owned(), pagos.to_string())].into(),
                });
            }

            let movimientos = repo.count_movimientos(id).await?;
            if movimientos > 0 {
                return Err(AppError::DependencyInUse {
                    code: "FACTURA_CON_MOVIMIENTOS",
                    message_key: "Conflict.Factura.ConMovimientos",
                    params: [("count".to_owned(), movimientos.to_string())].into(),
                });
            }

            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "factura eliminada");
        Ok(())
    }

    pub async fn pagos_de(&self, factura_id: Uuid) -> AppResult<Vec<PagoFacturaItem>> {
        let tx = self.uow.begin().await?;
        let result = tx.facturas().pagos_de(factura_id).await;
        let pagos = finish_read(tx, result).await?;
        Ok(pagos.iter().map(PagoFacturaItem::from).collect())
    }

    /// T-F07. Returns the whole invoice rather than the payment: the state and the balance both
    /// changed, and the screen would need a second round trip to learn how.
    pub async fn crear_pago(&self, input: PagoFacturaInput) -> AppResult<FacturaDetalle> {
        let hoy = self.hoy();
        validation::facturas::validate_pago(&input, &self.contexto_fecha(hoy))?;

        let now = self.clock.now_utc();
        let dias = self.dias_vencimiento();
        let tolerancia = self
            .settings
            .snapshot()
            .business
            .tolerancia_sobrepago_factura;

        let pago = PagoFactura {
            id: self.ids.new_id(),
            factura_id: input.factura_id,
            fecha: input.fecha,
            monto: input.monto,
            medio_pago: input.medio_pago.trim().to_owned(),
            audit: Audit::new(now),
        };

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.facturas();
            let factura = cargar_para_pago(repo, input.factura_id).await?;
            ensure_no_excede_saldo(&factura, pago.monto, None, tolerancia)?;

            repo.insert_pago(&pago).await?;
            recalcular(repo, input.factura_id, hoy, dias).await?;
            load_detalle(&*tx, input.factura_id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(
            factura = %detalle.id,
            monto = %pago.monto.to_decimal_string(),
            saldo = %detalle.saldo.to_decimal_string(),
            "pago registrado"
        );
        Ok(detalle)
    }

    /// T-F09.
    pub async fn actualizar_pago(
        &self,
        id: Uuid,
        input: PagoFacturaInput,
        row_version: &str,
    ) -> AppResult<FacturaDetalle> {
        let hoy = self.hoy();
        validation::facturas::validate_pago(&input, &self.contexto_fecha(hoy))?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();
        let dias = self.dias_vencimiento();
        let tolerancia = self
            .settings
            .snapshot()
            .business
            .tolerancia_sobrepago_factura;

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.facturas();
            let mut pago = repo
                .find_pago(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY_PAGO, id))?;

            let factura = cargar_para_pago(repo, pago.factura_id).await?;
            ensure_no_excede_saldo(&factura, input.monto, Some(pago.id), tolerancia)?;

            pago.fecha = input.fecha;
            pago.monto = input.monto;
            pago.medio_pago = input.medio_pago.trim().to_owned();
            pago.audit.touch(now);
            repo.update_pago(&pago, esperado).await?;

            recalcular(repo, pago.factura_id, hoy, dias).await?;
            load_detalle(&*tx, pago.factura_id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(factura = %detalle.id, pago = %id, "pago actualizado");
        Ok(detalle)
    }

    /// T-F08. Removing a payment can take the invoice back from `Pagada` to `PagadaParcial`, or
    /// all the way to `Emitida` once the last one is gone.
    pub async fn borrar_pago(&self, id: Uuid, row_version: &str) -> AppResult<FacturaDetalle> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();
        let hoy = self.hoy();
        let dias = self.dias_vencimiento();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.facturas();
            let pago = repo
                .find_pago(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY_PAGO, id))?;

            repo.soft_delete_pago(id, esperado, now).await?;
            recalcular(repo, pago.factura_id, hoy, dias).await?;
            load_detalle(&*tx, pago.factura_id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(factura = %detalle.id, pago = %id, "pago eliminado");
        Ok(detalle)
    }

    fn contexto_fecha(&self, hoy: NaiveDate) -> ContextoFecha {
        ContextoFecha::from_config(&self.settings.snapshot().validation, hoy)
    }
}

/// Doc 06 §4.1: the total is `subtotal + iva`, always. Whatever the form sent is discarded.
fn con_total_recalculado(mut input: FacturaInput) -> FacturaInput {
    if let Ok(total) = input.subtotal.checked_add(input.iva) {
        input.total = total;
    }
    input
}

async fn load_detalle(
    tx: &dyn Transaction,
    id: Uuid,
    hoy: NaiveDate,
    dias: u32,
) -> AppResult<FacturaDetalle> {
    let repo = tx.facturas();
    let factura = repo
        .find_con_pagos(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;
    let cliente_nombre = tx
        .clientes()
        .find_by_id(factura.cliente_id)
        .await?
        .map(|c| c.nombre)
        .unwrap_or_default();
    let libre =
        factura.pagos.iter().all(|p| p.audit.is_deleted) && repo.count_movimientos(id).await? == 0;
    FacturaDetalle::build(&factura, cliente_nombre, libre, hoy, dias)
}

async fn ensure_cliente_existe(tx: &dyn Transaction, id: Uuid) -> AppResult<()> {
    if tx.clientes().find_by_id(id).await?.is_none() {
        return Err(AppError::not_found("Cliente", id));
    }
    Ok(())
}

/// Reads the invoice and refuses up front when its state takes no money at all.
async fn cargar_para_pago(repo: &dyn FacturaRepository, id: Uuid) -> AppResult<Factura> {
    let factura = repo
        .find_con_pagos(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;

    if !factura.estado.admite_pagos() {
        return Err(AppError::Conflict {
            code: "FACTURA_NO_ADMITE_PAGOS",
            message_key: "State.Factura.NoAdmitePagos",
            params: [("estado".to_owned(), factura.estado.as_key().to_owned())].into(),
        });
    }
    Ok(factura)
}

/// INV-09. The legacy system had no such check: 10.000.000 could be imputed to an invoice of 1000.
fn ensure_no_excede_saldo(
    factura: &Factura,
    monto: Money,
    excluir: Option<Uuid>,
    tolerancia: Money,
) -> AppResult<()> {
    let ya_pagado = Money::try_sum(
        factura
            .pagos
            .iter()
            .filter(|p| !p.audit.is_deleted && Some(p.id) != excluir)
            .map(|p| p.monto),
    )?;
    let saldo = factura.total.checked_sub(ya_pagado)?;
    let limite = saldo.checked_add(tolerancia)?;

    if monto > limite {
        return Err(AppError::Conflict {
            code: "PAGO_EXCEDE_SALDO",
            message_key: "Validation.PagoFactura.ExcedeSaldo",
            params: [("saldo".to_owned(), saldo.to_decimal_string())].into(),
        });
    }
    Ok(())
}

/// Re-reads the invoice with its payments and writes back the derived state, in the transaction
/// the payment was written in. Only the state column is touched: bumping the row version here
/// would make the user's next save fail against a change they did not make.
async fn recalcular(
    repo: &dyn FacturaRepository,
    factura_id: Uuid,
    hoy: NaiveDate,
    dias: u32,
) -> AppResult<()> {
    let mut factura = repo
        .find_con_pagos(factura_id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, factura_id))?;
    let anterior = factura.estado;
    recalcular_estado_factura(&mut factura, hoy, dias)?;
    if factura.estado != anterior {
        repo.update_estado(factura_id, factura.estado).await?;
    }
    Ok(())
}
