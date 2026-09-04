use chrono::NaiveDate;
use certaro_domain::entities::{Audit, Factura};
use certaro_domain::{recalcular_estado_factura, EstadoFactura, StateMachine};
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::{ListQuery, LookupItem};
use crate::dtos::facturas::{FacturaDetalle, FacturaFiltroDto, FacturaInput, FacturaListItem};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::Transaction;
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;

use super::{FacturasService, ENTITY};

const SORTABLE: [&str; 6] = [
    "numero",
    "fecha",
    "fechaVencimiento",
    "clienteNombre",
    "total",
    "estado",
];

impl FacturasService {
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

            recalcular_estado_factura(&mut entity, hoy, dias)?;
            repo.update(&entity, esperado).await?;

            load_detalle(&*tx, id, hoy, dias).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "factura actualizada");
        Ok(detalle)
    }

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

            if pagos > 0 && matches!(destino, EstadoFactura::Anulada | EstadoFactura::Borrador) {
                return Err(AppError::Conflict {
                    code: "FACTURA_CON_PAGOS",
                    message_key: "State.Factura.TienePagos",
                    params: [("count".to_owned(), pagos.to_string())].into(),
                });
            }

            if destino == EstadoFactura::Emitida && !entity.total.is_positive() {
                return Err(AppError::conflict(
                    "FACTURA_TOTAL_NO_POSITIVO",
                    "State.Factura.RequiereTotalPositivo",
                ));
            }

            entity.estado = entity.estado.transition_to(destino)?;
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
}

fn con_total_recalculado(mut input: FacturaInput) -> FacturaInput {
    if let Ok(total) = input.subtotal.checked_add(input.iva) {
        input.total = total;
    }
    input
}

pub(crate) async fn load_detalle(
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
