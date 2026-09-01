//! Use cases of `obras`. See `docs/09-modulos-funcionales.md` §3.4.

use std::sync::Arc;

use certaro_domain::entities::{Audit, Obra};
use certaro_domain::{EstadoObra, EstadoTrabajo, Money, StateMachine};
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::{ListQuery, LookupItem};
use crate::dtos::obras::{ObraDetalle, ObraFiltroDto, ObraInput, ObraListItem};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{ObraRepository, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;

const ENTITY: &str = "Obra";

const SORTABLE: [&str; 5] = ["numero", "nombre", "clienteNombre", "estado", "createdAt"];

pub struct ObrasService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
}

impl ObrasService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, ids }
    }

    pub async fn list(
        &self,
        query: ListQuery<ObraFiltroDto>,
    ) -> AppResult<PagedResult<ObraListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let result = tx
            .obras()
            .search(&filtro, page, sort_by, query.sort_dir)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(ObraListItem::from))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<ObraDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(tx.obras(), id).await;
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
            .obras()
            .lookup(cliente_id, texto.as_deref(), limite.unwrap_or(50))
            .await;
        let obras = finish_read(tx, result).await?;
        Ok(obras
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
        let result = tx.obras().siguiente_numero().await;
        finish_read(tx, result).await
    }

    pub async fn create(&self, input: ObraInput) -> AppResult<ObraDetalle> {
        validation::obras::validate(&input)?;

        let now = self.clock.now_utc();
        let entity = Obra {
            id: self.ids.new_id(),
            numero: input.numero,
            nombre: input.nombre.trim().to_owned(),
            direccion: normalise(input.direccion),
            localidad: normalise(input.localidad),
            cliente_id: input.cliente_id,
            estado: EstadoObra::Activa,
            audit: Audit::new(now),
        };

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.obras();
            ensure_cliente_existe(&*tx, entity.cliente_id).await?;
            ensure_numero_libre(repo, entity.numero, None).await?;
            repo.insert(&entity).await?;
            let cliente_nombre = nombre_de_cliente(&*tx, entity.cliente_id).await?;
            Ok(ObraDetalle::build(&entity, cliente_nombre, 0, Money::ZERO))
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, numero = detalle.numero, "obra creada");
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: ObraInput,
        row_version: &str,
    ) -> AppResult<ObraDetalle> {
        validation::obras::validate(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.obras();
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

        info!(id = %detalle.id, "obra actualizada");
        Ok(detalle)
    }

    /// The only way the state changes. `cascada` is only consulted when closing the site: with
    /// open jobs and no cascade the call is refused so the interface can ask the question.
    pub async fn transition(
        &self,
        id: Uuid,
        destino: EstadoObra,
        row_version: &str,
        cascada: bool,
    ) -> AppResult<ObraDetalle> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.obras();
            let mut entity = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let cierre = matches!(destino, EstadoObra::Finalizada | EstadoObra::Cancelada);
            let abiertos = if cierre {
                repo.trabajos_abiertos(id).await?
            } else {
                Vec::new()
            };

            if !abiertos.is_empty() && !cascada {
                return Err(AppError::Conflict {
                    code: "OBRA_CON_TRABAJOS_ABIERTOS",
                    message_key: "State.Obra.TieneTrabajosAbiertos",
                    params: [("count".to_owned(), abiertos.len().to_string())].into(),
                });
            }

            entity.estado = entity.estado.transition_to(destino)?;
            entity.audit.touch(now);
            repo.update(&entity, esperado).await?;

            // Closing the site closes what is still open inside it, in the same transaction, so
            // there is no window where a finished site holds a job in progress.
            let destino_trabajo = if destino == EstadoObra::Cancelada {
                EstadoTrabajo::Cancelado
            } else {
                EstadoTrabajo::Finalizado
            };
            for mut trabajo in abiertos {
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

        info!(id = %detalle.id, estado = %detalle.estado.actual, "obra transicionada");
        Ok(detalle)
    }

    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.obras();
            repo.find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let trabajos = repo.count_trabajos(id).await?;
            if trabajos > 0 {
                return Err(AppError::DependencyInUse {
                    code: "OBRA_CON_TRABAJOS",
                    message_key: "Conflict.Obra.ConTrabajos",
                    params: [("count".to_owned(), trabajos.to_string())].into(),
                });
            }

            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "obra eliminada");
        Ok(())
    }
}

async fn load_detalle(repo: &dyn ObraRepository, id: Uuid) -> AppResult<ObraDetalle> {
    repo.find_detalle(id)
        .await?
        .map(ObraDetalle::from)
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
    repo: &dyn ObraRepository,
    numero: i32,
    excluir: Option<Uuid>,
) -> AppResult<()> {
    if repo.numero_ocupado(numero, excluir).await? {
        return Err(AppError::Conflict {
            code: "OBRA_NUMERO_DUPLICADO",
            message_key: "Validation.Obra.NumeroDuplicado",
            params: [("numero".to_owned(), numero.to_string())].into(),
        });
    }
    Ok(())
}
