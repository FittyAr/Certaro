//! Use cases of `trabajos`. See `docs/09-modulos-funcionales.md` §3.5.
//!
//! The legacy screen existed, had shortcuts bound and was never registered in the navigation, so
//! it was unreachable; and its customer filter read a denormalised column on the job. Here the
//! customer is always resolved through the site.

use std::sync::Arc;

use certaro_domain::entities::{Audit, Trabajo};
use certaro_domain::{EstadoObra, EstadoTrabajo, StateMachine};
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::{ListQuery, LookupItem};
use crate::dtos::trabajos::{TrabajoDetalle, TrabajoFiltroDto, TrabajoInput, TrabajoListItem};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{TrabajoRepository, Transaction, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort};
use crate::result::AppResult;
use crate::use_cases::shared::{checked_sort, finish_read, finish_write, parse_row_version};
use crate::validation;

const ENTITY: &str = "Trabajo";

const SORTABLE: [&str; 6] = [
    "descripcion",
    "fechaInicio",
    "fechaFin",
    "presupuesto",
    "estado",
    "obraNumero",
];

pub struct TrabajosService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
}

impl TrabajosService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, ids }
    }

    pub async fn list(
        &self,
        query: ListQuery<TrabajoFiltroDto>,
    ) -> AppResult<PagedResult<TrabajoListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let result = tx
            .trabajos()
            .search(&filtro, page, sort_by, query.sort_dir)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(TrabajoListItem::from))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<TrabajoDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(tx.trabajos(), id).await;
        finish_read(tx, loaded).await
    }

    pub async fn lookup(
        &self,
        obra_id: Option<Uuid>,
        texto: Option<String>,
        limite: Option<u64>,
    ) -> AppResult<Vec<LookupItem>> {
        let tx = self.uow.begin().await?;
        let result = tx
            .trabajos()
            .lookup(obra_id, texto.as_deref(), limite.unwrap_or(50))
            .await;
        let trabajos = finish_read(tx, result).await?;
        Ok(trabajos
            .into_iter()
            .map(|t| {
                LookupItem::new(t.id, t.descripcion).with_meta("obraId", t.obra_id.to_string())
            })
            .collect())
    }

    pub async fn create(&self, input: TrabajoInput) -> AppResult<TrabajoDetalle> {
        validation::trabajos::validate(&input)?;

        let now = self.clock.now_utc();
        let entity = Trabajo {
            id: self.ids.new_id(),
            obra_id: input.obra_id,
            descripcion: input.descripcion.trim().to_owned(),
            fecha_inicio: input.fecha_inicio,
            fecha_fin: input.fecha_fin,
            presupuesto: input.presupuesto,
            estado: EstadoTrabajo::Presupuestado,
            audit: Audit::new(now),
        };

        let tx = self.uow.begin().await?;
        let outcome = async {
            // T-T01: a cancelled site takes no new work. Everything else does, because quoting a
            // job on a paused site is how a site gets restarted.
            let obra = cargar_obra(&*tx, entity.obra_id).await?;
            if obra.estado == EstadoObra::Cancelada {
                return Err(obra_no_disponible(obra.estado));
            }

            tx.trabajos().insert(&entity).await?;
            load_detalle(tx.trabajos(), entity.id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, obra = %detalle.obra_id, "trabajo creado");
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: TrabajoInput,
        row_version: &str,
    ) -> AppResult<TrabajoDetalle> {
        validation::trabajos::validate(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.trabajos();
            let mut entity = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            cargar_obra(&*tx, input.obra_id).await?;

            entity.obra_id = input.obra_id;
            entity.descripcion = input.descripcion.trim().to_owned();
            entity.fecha_inicio = input.fecha_inicio;
            entity.fecha_fin = input.fecha_fin;
            entity.presupuesto = input.presupuesto;
            entity.audit.touch(now);

            repo.update(&entity, esperado).await?;
            load_detalle(repo, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "trabajo actualizado");
        Ok(detalle)
    }

    /// The only way the state changes. `forzar` is reserved for the incomplete-certification
    /// warning of doc 08 §4.4, which arrives with the certification module.
    pub async fn transition(
        &self,
        id: Uuid,
        destino: EstadoTrabajo,
        row_version: &str,
        _forzar: bool,
    ) -> AppResult<TrabajoDetalle> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.trabajos();
            let mut entity = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            // Starting, resuming and reopening need a running site; closing never does, because
            // shutting things down has to stay possible whatever state the site is in.
            if EstadoTrabajo::exige_obra_activa(destino, entity.estado) {
                let obra = cargar_obra(&*tx, entity.obra_id).await?;
                if !obra.esta_activa() {
                    return Err(obra_no_disponible(obra.estado));
                }
            }

            let anterior = entity.estado;
            entity.estado = entity.estado.transition_to(destino)?;

            match destino {
                EstadoTrabajo::Finalizado if entity.fecha_fin.is_none() => {
                    entity.fecha_fin = Some(now.date_naive());
                }
                // Reopening clears the end date: a job in progress has not ended.
                EstadoTrabajo::EnProceso if anterior == EstadoTrabajo::Finalizado => {
                    entity.fecha_fin = None;
                }
                _ => {}
            }

            entity.audit.touch(now);
            repo.update(&entity, esperado).await?;
            load_detalle(repo, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, estado = %detalle.estado.actual, "trabajo transicionado");
        Ok(detalle)
    }

    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.trabajos();
            repo.find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            for (count, code, key) in [
                (
                    repo.count_ordenes(id).await?,
                    "TRABAJO_CON_ORDENES",
                    "Conflict.Trabajo.ConOrdenes",
                ),
                (
                    repo.count_movimientos(id).await?,
                    "TRABAJO_CON_MOVIMIENTOS",
                    "Conflict.Trabajo.ConMovimientos",
                ),
            ] {
                if count > 0 {
                    return Err(AppError::DependencyInUse {
                        code,
                        message_key: key,
                        params: [("count".to_owned(), count.to_string())].into(),
                    });
                }
            }

            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "trabajo eliminado");
        Ok(())
    }
}

async fn load_detalle(repo: &dyn TrabajoRepository, id: Uuid) -> AppResult<TrabajoDetalle> {
    let row = repo
        .find_detalle(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;
    let libre = repo.count_ordenes(id).await? == 0 && repo.count_movimientos(id).await? == 0;
    Ok(TrabajoDetalle::build(&row, libre))
}

async fn cargar_obra(tx: &dyn Transaction, id: Uuid) -> AppResult<certaro_domain::entities::Obra> {
    tx.obras()
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::not_found("Obra", id))
}

fn obra_no_disponible(estado: EstadoObra) -> AppError {
    AppError::Conflict {
        code: "TRABAJO_OBRA_NO_ACTIVA",
        message_key: "State.Trabajo.ObraNoActiva",
        // The value is the state **key**, not text: the frontend translates before interpolating.
        params: [("estadoObra".to_owned(), estado.as_key().to_owned())].into(),
    }
}
