//! Use cases of `tipos_movimiento`. See `docs/09-modulos-funcionales.md` and
//! `docs/11-contratos-tauri.md` §5.11.
//!
//! This module is the reference shape for every other aggregate: validate the input, open a
//! transaction, check the invariants that need the database, write through the port, commit, log.

use std::sync::Arc;

use certaro_domain::entities::{Audit, TipoMovimiento};
use tracing::info;
use uuid::Uuid;

use crate::dtos::common::{ListQuery, LookupItem};
use crate::dtos::tipos_movimiento::{
    TipoMovimientoDetalle, TipoMovimientoFiltroDto, TipoMovimientoInput, TipoMovimientoListItem,
};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::UnitOfWork;
use crate::ports::{ClockPort, IdGeneratorPort};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;

const ENTITY: &str = "TipoMovimiento";

/// Columns the frontend is allowed to sort by. An arbitrary name coming from the interface would
/// end up in an `ORDER BY`, so the list is closed rather than validated by pattern.
const SORTABLE: [&str; 4] = ["nombre", "esIngreso", "movimientosCount", "createdAt"];

pub struct TiposMovimientoService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
}

impl TiposMovimientoService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, ids }
    }

    pub async fn list(
        &self,
        query: ListQuery<TipoMovimientoFiltroDto>,
    ) -> AppResult<PagedResult<TipoMovimientoListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let result = tx
            .tipos_movimiento()
            .search(&filtro, page, sort_by, query.sort_dir)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(TipoMovimientoListItem::from))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<TipoMovimientoDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(tx.tipos_movimiento(), id).await;
        finish_read(tx, loaded).await
    }

    pub async fn lookup(
        &self,
        texto: Option<String>,
        limite: Option<u64>,
    ) -> AppResult<Vec<LookupItem>> {
        let tx = self.uow.begin().await?;
        let result = tx
            .tipos_movimiento()
            .lookup(texto.as_deref(), limite.unwrap_or(50))
            .await;
        let tipos = finish_read(tx, result).await?;
        Ok(tipos
            .into_iter()
            .map(|t| {
                LookupItem::new(t.id, t.nombre)
                    .with_meta("esIngreso", t.es_ingreso.to_string())
                    .with_meta("esSistema", t.es_sistema.to_string())
            })
            .collect())
    }

    pub async fn create(&self, input: TipoMovimientoInput) -> AppResult<TipoMovimientoDetalle> {
        validation::tipos_movimiento::validate(&input)?;

        let now = self.clock.now_utc();
        let entity = TipoMovimiento {
            id: self.ids.new_id(),
            nombre: input.nombre.trim().to_owned(),
            descripcion: normalise(input.descripcion),
            es_ingreso: input.es_ingreso,
            // Only the seed creates system rows; the interface cannot.
            es_sistema: false,
            audit: Audit::new(now),
        };

        let tx = self.uow.begin().await?;
        let outcome = async {
            ensure_nombre_libre(tx.tipos_movimiento(), &entity.nombre, None).await?;
            tx.tipos_movimiento().insert(&entity).await?;
            Ok(TipoMovimientoDetalle::build(&entity, 0))
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, nombre = %detalle.nombre, "tipo de movimiento creado");
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: TipoMovimientoInput,
        row_version: &str,
    ) -> AppResult<TipoMovimientoDetalle> {
        validation::tipos_movimiento::validate(&input)?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.tipos_movimiento();
            let mut entity = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let nombre = input.nombre.trim().to_owned();
            ensure_nombre_libre(repo, &nombre, Some(id)).await?;

            // A system row keeps its sign: the historical balance was computed with it, and
            // flipping it now would silently rewrite every past total.
            if entity.es_de_sistema_protegido() && entity.es_ingreso != input.es_ingreso {
                return Err(AppError::conflict(
                    "TIPO_MOVIMIENTO_SISTEMA",
                    "Conflict.TipoMovimiento.SignoDeSistema",
                ));
            }

            entity.nombre = nombre;
            entity.descripcion = normalise(input.descripcion);
            entity.es_ingreso = input.es_ingreso;
            entity.audit.touch(now);

            repo.update(&entity, esperado).await?;
            let usados = repo.count_movimientos(id).await?;
            Ok(TipoMovimientoDetalle::build(&entity, usados))
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "tipo de movimiento actualizado");
        Ok(detalle)
    }

    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.tipos_movimiento();
            let entity = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            if entity.es_de_sistema_protegido() {
                return Err(AppError::conflict(
                    "TIPO_MOVIMIENTO_SISTEMA",
                    "Conflict.TipoMovimiento.EsDeSistema",
                ));
            }

            let usados = repo.count_movimientos(id).await?;
            if usados > 0 {
                return Err(AppError::DependencyInUse {
                    code: "TIPO_MOVIMIENTO_EN_USO",
                    message_key: "Conflict.TipoMovimiento.EnUso",
                    params: [("count".to_owned(), usados.to_string())].into(),
                });
            }

            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "tipo de movimiento eliminado");
        Ok(())
    }
}

async fn load_detalle(
    repo: &dyn crate::ports::TipoMovimientoRepository,
    id: Uuid,
) -> AppResult<TipoMovimientoDetalle> {
    let entity = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;
    let usados = repo.count_movimientos(id).await?;
    Ok(TipoMovimientoDetalle::build(&entity, usados))
}

async fn ensure_nombre_libre(
    repo: &dyn crate::ports::TipoMovimientoRepository,
    nombre: &str,
    excluir: Option<Uuid>,
) -> AppResult<()> {
    if repo.find_by_nombre(nombre, excluir).await?.is_some() {
        return Err(AppError::Conflict {
            code: "TIPO_MOVIMIENTO_NOMBRE_DUPLICADO",
            message_key: "Conflict.TipoMovimiento.NombreDuplicado",
            params: [("nombre".to_owned(), nombre.to_owned())].into(),
        });
    }
    Ok(())
}
