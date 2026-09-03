//! Use cases of `proyectos`. See `docs/09-modulos-funcionales.md` §3.4.

use std::sync::Arc;

use certaro_domain::entities::{Audit, Proyecto};
use certaro_domain::{EstadoProyecto, EstadoTrabajo, Money, StateMachine};
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::{ListQuery, LookupItem};
use crate::dtos::proyectos::{ProyectoDetalle, ProyectoFiltroDto, ProyectoInput, ProyectoListItem};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{ProyectoRepository, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;

const ENTITY: &str = "Proyecto";

const SORTABLE: [&str; 5] = ["numero", "nombre", "clienteNombre", "estado", "createdAt"];

pub struct ProyectosService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
}

impl ProyectosService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, ids }
    }

    pub async fn list(
        &self,
        query: ListQuery<ProyectoFiltroDto>,
    ) -> AppResult<PagedResult<ProyectoListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let result = tx
            .proyectos()
            .search(&filtro, page, sort_by, query.sort_dir)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(ProyectoListItem::from))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<ProyectoDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(tx.proyectos(), id).await;
        finish_read(tx, loaded).await
    }

    pub async fn lookup(
        &self,
        cliente_id: Option<Uuid>,
        texto: Option<String>,
        limite: Option<u64>,
    ) -> AppResult<Vec<LookupItem>> {
        let tx = self.uow.begin().await?;
        let result = tx
            .proyectos()
            .lookup(cliente_id, texto.as_deref(), limite.unwrap_or(50))
            .await;
        let proyectos = finish_read(tx, result).await?;
        Ok(proyectos
            .into_iter()
            .map(|o| {
                LookupItem::new(o.id, format!("{} - {}", o.numero, o.nombre))
                    .with_meta("numero", o.numero.to_string())
                    .with_meta("clienteId", o.cliente_id.to_string())
            })
            .collect())
    }

    /// `MAX(numero) + 1` over every row, deleted ones included. The legacy version took the
    /// maximum of the page loaded in memory, which was paged and excluded deletions, so it happily
    /// proposed a number that was already taken.
    pub async fn siguiente_numero(&self) -> AppResult<i32> {
        let tx = self.uow.begin().await?;
        let result = tx.proyectos().siguiente_numero().await;
        finish_read(tx, result).await
    }

    pub async fn create(&self, input: ProyectoInput) -> AppResult<ProyectoDetalle> {
        validation::proyectos::validate(&input)?;

        let now = self.clock.now_utc();
        let entity = Proyecto {
            id: self.ids.new_id(),
            numero: input.numero,
            nombre: input.nombre.trim().to_owned(),
            direccion: normalise(input.direccion),
            localidad: normalise(input.localidad),
            cliente_id: input.cliente_id,
            estado: EstadoProyecto::Activa,
            audit: Audit::new(now),
        };

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.proyectos();
            ensure_cliente_existe(&*tx, entity.cliente_id).await?;
            ensure_numero_libre(repo, entity.numero, None).await?;
            repo.insert(&entity).await?;
            let cliente_nombre = nombre_de_cliente(&*tx, entity.cliente_id).await?;
            Ok(ProyectoDetalle::build(&entity, cliente_nombre, 0, Money::ZERO))
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, numero = detalle.numero, "proyecto creada");
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: ProyectoInput,
        row_version: &str,
    ) -> AppResult<ProyectoDetalle> {
        validation::proyectos::validate(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.proyectos();
            let mut entity = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            ensure_cliente_existe(&*tx, input.cliente_id).await?;
            ensure_numero_libre(repo, input.numero, Some(id)).await?;

            entity.numero = input.numero;
            entity.nombre = input.nombre.trim().to_owned();
            entity.direccion = normalise(input.direccion);
            entity.localidad = normalise(input.localidad);
            entity.cliente_id = input.cliente_id;
            entity.audit.touch(now);

            repo.update(&entity, esperado).await?;
            load_detalle(repo, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "proyecto actualizada");
        Ok(detalle)
    }

    /// The only way the state changes. `cascada` is only consulted when closing the site: with
    /// open jobs and no cascade the call is refused so the interface can ask the question.
    pub async fn transition(
        &self,
        id: Uuid,
        destino: EstadoProyecto,
        row_version: &str,
        cascada: bool,
    ) -> AppResult<ProyectoDetalle> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.proyectos();
            let mut entity = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let cierre = matches!(destino, EstadoProyecto::Finalizada | EstadoProyecto::Cancelada);
            let abiertos = if cierre {
                repo.trabajos_abiertos(id).await?
            } else {
                Vec::new()
            };

            if !abiertos.is_empty() && !cascada {
                return Err(AppError::Conflict {
                    code: "PROYECTO_CON_TRABAJOS_ABIERTOS",
                    message_key: "State.Proyecto.TieneTrabajosAbiertos",
                    params: [("count".to_owned(), abiertos.len().to_string())].into(),
                });
            }

            entity.estado = entity.estado.transition_to(destino)?;
            entity.audit.touch(now);
            repo.update(&entity, esperado).await?;

            for mut trabajo in abiertos {
                let destino_trabajo = if destino == EstadoProyecto::Cancelada
                    || trabajo.estado == EstadoTrabajo::Presupuestado
                {
                    EstadoTrabajo::Cancelado
                } else {
                    EstadoTrabajo::Finalizado
                };
                trabajo.estado = trabajo.estado.transition_to(destino_trabajo)?;
                if trabajo.fecha_fin.is_none() {
                    trabajo.fecha_fin = Some(now.date_naive());
                }
                let version = trabajo.audit.row_version;
                trabajo.audit.touch(now);
                tx.trabajos().update(&trabajo, version).await?;
            }

            load_detalle(repo, id).await
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, estado = %detalle.estado.actual, "proyecto transicionada");
        Ok(detalle)
    }

    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.proyectos();
            repo.find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let trabajos = repo.count_trabajos(id).await?;
            if trabajos > 0 {
                return Err(AppError::DependencyInUse {
                    code: "PROYECTO_CON_TRABAJOS",
                    message_key: "Conflict.Proyecto.ConTrabajos",
                    params: [("count".to_owned(), trabajos.to_string())].into(),
                });
            }

            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "proyecto eliminada");
        Ok(())
    }
}

async fn load_detalle(repo: &dyn ProyectoRepository, id: Uuid) -> AppResult<ProyectoDetalle> {
    repo.find_detalle(id)
        .await?
        .map(ProyectoDetalle::from)
        .ok_or_else(|| AppError::not_found(ENTITY, id))
}

async fn ensure_cliente_existe(tx: &dyn crate::ports::Transaction, id: Uuid) -> AppResult<()> {
    if tx.clientes().find_by_id(id).await?.is_none() {
        return Err(AppError::not_found("Cliente", id));
    }
    Ok(())
}

async fn nombre_de_cliente(tx: &dyn crate::ports::Transaction, id: Uuid) -> AppResult<String> {
    Ok(tx
        .clientes()
        .find_by_id(id)
        .await?
        .map(|c| c.nombre)
        .unwrap_or_default())
}

/// INV-06. A deleted site keeps its number reserved, which is why the unique index is not filtered
/// by `is_deleted`; the message has to say so or the refusal looks like a bug.
async fn ensure_numero_libre(
    repo: &dyn ProyectoRepository,
    numero: i32,
    excluir: Option<Uuid>,
) -> AppResult<()> {
    if repo.numero_ocupado(numero, excluir).await? {
        return Err(AppError::Conflict {
            code: "PROYECTO_NUMERO_DUPLICADO",
            message_key: "Validation.Proyecto.NumeroDuplicado",
            params: [("numero".to_owned(), numero.to_string())].into(),
        });
    }
    Ok(())
}
