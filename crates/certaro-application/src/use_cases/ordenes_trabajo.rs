//! Use cases of `ordenes_trabajo`. See `docs/09-modulos-funcionales.md` §3.6.
//!
//! The order and its items are one aggregate, written in a single transaction, with one asymmetry:
//! an item that already appears in a certificate cannot be dropped by the form. The certified line
//! points at it, and letting the sheet delete it would leave the history without an anchor.

use std::sync::Arc;

use certaro_domain::entities::{Audit, OrdenTrabajo, OrdenTrabajoItem};
use certaro_domain::Decimal4;
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::LookupItem;
use crate::dtos::ordenes_trabajo::{
    OrdenTrabajoDetalle, OrdenTrabajoInput, OrdenTrabajoItemInput, OrdenTrabajoListItem,
};
use crate::error::AppError;
use crate::ports::repositories::{OrdenTrabajoRepository, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort};
use crate::result::AppResult;
use crate::use_cases::shared::{finish_read, finish_write, normalise, parse_row_version};
use crate::validation;

const ENTITY: &str = "OrdenTrabajo";

pub struct OrdenesTrabajoService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
}

impl OrdenesTrabajoService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, ids }
    }

    /// Every order of a job. Not paged: a job has a handful of sheets, not thousands.
    pub async fn de_trabajo(&self, trabajo_id: Uuid) -> AppResult<Vec<OrdenTrabajoListItem>> {
        let tx = self.uow.begin().await?;
        let result = tx.ordenes_trabajo().de_trabajo(trabajo_id).await;
        let rows = finish_read(tx, result).await?;
        rows.iter().map(OrdenTrabajoListItem::build).collect()
    }

    pub async fn get(&self, id: Uuid) -> AppResult<OrdenTrabajoDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(tx.ordenes_trabajo(), id).await;
        finish_read(tx, loaded).await
    }

    pub async fn lookup(
        &self,
        trabajo_id: Option<Uuid>,
        texto: Option<String>,
        limite: Option<u64>,
    ) -> AppResult<Vec<LookupItem>> {
        let tx = self.uow.begin().await?;
        let result = tx
            .ordenes_trabajo()
            .lookup(trabajo_id, texto.as_deref(), limite.unwrap_or(50))
            .await;
        let ordenes = finish_read(tx, result).await?;
        Ok(ordenes
            .into_iter()
            .map(|o| LookupItem::new(o.id, o.titulo))
            .collect())
    }

    pub async fn create(&self, input: OrdenTrabajoInput) -> AppResult<OrdenTrabajoDetalle> {
        validation::ordenes_trabajo::validate(&input)?;
        // On a new order nothing has been certified, so the accumulated history is zero and the
        // per-item shape check already covers the ceiling.
        validar_acumulados(&input.items, |_| Decimal4::ZERO)?;

        let now = self.clock.now_utc();
        let id = self.ids.new_id();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.ordenes_trabajo();
            tx.trabajos()
                .find_by_id(input.trabajo_id)
                .await?
                .ok_or_else(|| AppError::not_found("Trabajo", input.trabajo_id))?;

            let mut orden = build_orden(id, &input, now);
            orden.items = input
                .items
                .iter()
                .enumerate()
                .map(|(i, item)| self.build_item(item, id, i, Decimal4::ZERO, now))
                .collect();

            repo.insert(&orden).await?;
            for item in &orden.items {
                repo.insert_item(item).await?;
            }
            load_detalle(repo, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, items = detalle.items.len(), "orden de trabajo creada");
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: OrdenTrabajoInput,
        row_version: &str,
    ) -> AppResult<OrdenTrabajoDetalle> {
        validation::ordenes_trabajo::validate(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.ordenes_trabajo();
            let existente = repo
                .find_con_items(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            // The history each line carries is not in the form, so the ceiling is checked against
            // what is already stored.
            validar_acumulados(&input.items, |item_id| {
                item_id
                    .and_then(|item_id| existente.items.iter().find(|i| i.id == item_id))
                    .map_or(Decimal4::ZERO, |i| i.porcentaje_anterior)
            })?;

            let certificados = repo.items_certificados(id).await?;

            let mut orden = build_orden(id, &input, now);
            orden.audit = existente.audit.clone();
            orden.audit.touch(now);
            orden.numero_certificado = existente.numero_certificado.clone();
            repo.update(&orden, esperado).await?;

            let mut conservar = Vec::with_capacity(input.items.len());
            for (i, entrante) in input.items.iter().enumerate() {
                let previo = entrante
                    .id
                    .and_then(|item_id| existente.items.iter().find(|it| it.id == item_id));
                let anterior = previo.map_or(Decimal4::ZERO, |p| p.porcentaje_anterior);
                let mut item = self.build_item(entrante, id, i, anterior, now);
                match previo {
                    Some(previo) => {
                        item.id = previo.id;
                        item.audit = previo.audit.clone();
                        item.audit.touch(now);
                        repo.update_item(&item).await?;
                    }
                    None => {
                        // A stale id is a stale form, not a target: inserting under a fresh id
                        // keeps the line the user typed instead of dropping it.
                        item.id = self.ids.new_id();
                        repo.insert_item(&item).await?;
                    }
                }
                conservar.push(item.id);
            }

            // A certified line survives the form even when it is no longer in the grid.
            for certificado in &certificados {
                if !conservar.contains(certificado) {
                    return Err(AppError::Conflict {
                        code: "ITEM_CERTIFICADO",
                        message_key: "Conflict.OrdenTrabajo.ItemCertificado",
                        params: Default::default(),
                    });
                }
            }

            repo.soft_delete_items_excepto(id, &conservar, now).await?;
            load_detalle(repo, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, items = detalle.items.len(), "orden de trabajo actualizada");
        Ok(detalle)
    }

    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.ordenes_trabajo();
            repo.find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let certificados = repo.count_certificados(id).await?;
            if certificados > 0 {
                return Err(AppError::DependencyInUse {
                    code: "ORDEN_CON_CERTIFICADOS",
                    message_key: "Conflict.OrdenTrabajo.ConCertificados",
                    params: [("count".to_owned(), certificados.to_string())].into(),
                });
            }

            repo.soft_delete_items_excepto(id, &[], now).await?;
            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "orden de trabajo eliminada");
        Ok(())
    }

    fn build_item(
        &self,
        input: &OrdenTrabajoItemInput,
        orden_trabajo_id: Uuid,
        posicion: usize,
        porcentaje_anterior: Decimal4,
        now: chrono::DateTime<chrono::Utc>,
    ) -> OrdenTrabajoItem {
        OrdenTrabajoItem {
            id: input.id.unwrap_or_else(|| self.ids.new_id()),
            orden_trabajo_id,
            descripcion: input.descripcion.trim().to_owned(),
            unidad: normalise(Some(input.unidad.clone())).unwrap_or_else(|| "u".to_owned()),
            cantidad: input.cantidad,
            precio_unitario: input.precio_unitario,
            porcentaje_anterior,
            porcentaje_actual: input.porcentaje_actual,
            ejecutado: input.ejecutado,
            nota: normalise(input.nota.clone()),
            // The position comes from the list order, not from a number the form sends: the user
            // reorders by dragging and the printed sheet has to follow.
            orden: i32::try_from(posicion).unwrap_or(i32::MAX),
            audit: Audit::new(now),
        }
    }
}

/// Applies V-09's accumulated ceiling with the history the input does not carry.
fn validar_acumulados(
    items: &[OrdenTrabajoItemInput],
    anterior_de: impl Fn(Option<Uuid>) -> Decimal4,
) -> AppResult<()> {
    let errores: Vec<_> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            validation::ordenes_trabajo::validar_acumulado(
                i,
                anterior_de(item.id),
                item.porcentaje_actual,
            )
        })
        .collect();

    if errores.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(errores))
    }
}

fn build_orden(
    id: Uuid,
    input: &OrdenTrabajoInput,
    now: chrono::DateTime<chrono::Utc>,
) -> OrdenTrabajo {
    OrdenTrabajo {
        id,
        trabajo_id: input.trabajo_id,
        titulo: input.titulo.trim().to_owned(),
        numero_certificado: None,
        fecha: input.fecha,
        observaciones: normalise(input.observaciones.clone()),
        ajuste_uocra_porcentaje: input.ajuste_uocra_porcentaje,
        otros_descuentos: input.otros_descuentos,
        items: Vec::new(),
        audit: Audit::new(now),
    }
}

async fn load_detalle(
    repo: &dyn OrdenTrabajoRepository,
    id: Uuid,
) -> AppResult<OrdenTrabajoDetalle> {
    let row = repo
        .find_detalle(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;
    let certificados = repo.items_certificados(id).await?;
    OrdenTrabajoDetalle::build(&row, &certificados)
}
