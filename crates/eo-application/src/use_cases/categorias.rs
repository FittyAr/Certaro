//! Use cases of `categorias`. See `docs/09-modulos-funcionales.md` §3.13.

use std::sync::Arc;

use eo_domain::entities::{Audit, Categoria};
use tracing::info;
use uuid::Uuid;

use crate::dtos::categorias::{
    CategoriaDetalle, CategoriaFiltroDto, CategoriaInput, CategoriaListItem,
};
use crate::dtos::common::{ListQuery, LookupItem};
use crate::error::AppError;
use crate::paging::PagedResult;
use crate::ports::repositories::{CategoriaRepository, UnitOfWork};
use crate::ports::{ClockPort, IdGeneratorPort};
use crate::result::AppResult;
use crate::use_cases::shared::{
    checked_sort, finish_read, finish_write, normalise, parse_row_version,
};
use crate::validation;

const ENTITY: &str = "Categoria";

const SORTABLE: [&str; 4] = ["nombre", "movimientosCount", "hijasCount", "createdAt"];

pub struct CategoriasService {
    uow: Arc<dyn UnitOfWork>,
    clock: Arc<dyn ClockPort>,
    ids: Arc<dyn IdGeneratorPort>,
}

impl CategoriasService {
    pub fn new(
        uow: Arc<dyn UnitOfWork>,
        clock: Arc<dyn ClockPort>,
        ids: Arc<dyn IdGeneratorPort>,
    ) -> Self {
        Self { uow, clock, ids }
    }

    pub async fn list(
        &self,
        query: ListQuery<CategoriaFiltroDto>,
    ) -> AppResult<PagedResult<CategoriaListItem>> {
        let sort_by = checked_sort(query.sort_by.as_deref(), &SORTABLE)?;
        let page = query.page_request();
        page.validate()?;
        let filtro = query.filtro.into();

        let tx = self.uow.begin().await?;
        let result = tx
            .categorias()
            .search(&filtro, page, sort_by, query.sort_dir)
            .await;
        let page = finish_read(tx, result).await?;
        Ok(page.map(CategoriaListItem::from))
    }

    pub async fn get(&self, id: Uuid) -> AppResult<CategoriaDetalle> {
        let tx = self.uow.begin().await?;
        let loaded = load_detalle(tx.categorias(), id).await;
        finish_read(tx, loaded).await
    }

    pub async fn lookup(
        &self,
        texto: Option<String>,
        limite: Option<u64>,
    ) -> AppResult<Vec<LookupItem>> {
        let tx = self.uow.begin().await?;
        let result = tx
            .categorias()
            .lookup(texto.as_deref(), limite.unwrap_or(50))
            .await;
        let categorias = finish_read(tx, result).await?;
        Ok(categorias
            .into_iter()
            .map(|c| {
                let item = LookupItem::new(c.id, c.nombre);
                match c.color_hex {
                    Some(color) => item.with_meta("colorHex", color),
                    None => item,
                }
            })
            .collect())
    }

    pub async fn create(&self, input: CategoriaInput) -> AppResult<CategoriaDetalle> {
        validation::categorias::validate(&input, None)?;

        let now = self.clock.now_utc();
        let entity = Categoria {
            id: self.ids.new_id(),
            nombre: input.nombre.trim().to_owned(),
            descripcion: normalise(input.descripcion),
            color_hex: normalise(input.color_hex).map(|c| c.to_uppercase()),
            icono: normalise(input.icono),
            categoria_padre_id: input.categoria_padre_id,
            audit: Audit::new(now),
        };

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.categorias();
            ensure_padre_existe(repo, entity.categoria_padre_id).await?;
            ensure_nombre_libre(repo, &entity.nombre, entity.categoria_padre_id, None).await?;
            repo.insert(&entity).await?;
            Ok(CategoriaDetalle::build(&entity, 0, 0))
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, nombre = %detalle.nombre, "categoría creada");
        Ok(detalle)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: CategoriaInput,
        row_version: &str,
    ) -> AppResult<CategoriaDetalle> {
        validation::categorias::validate(&input, Some(id))?;
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.categorias();
            let mut entity = repo
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            let nombre = input.nombre.trim().to_owned();
            ensure_padre_existe(repo, input.categoria_padre_id).await?;
            ensure_sin_ciclo(repo, id, input.categoria_padre_id).await?;
            ensure_nombre_libre(repo, &nombre, input.categoria_padre_id, Some(id)).await?;

            entity.nombre = nombre;
            entity.descripcion = normalise(input.descripcion);
            entity.color_hex = normalise(input.color_hex).map(|c| c.to_uppercase());
            entity.icono = normalise(input.icono);
            entity.categoria_padre_id = input.categoria_padre_id;
            entity.audit.touch(now);

            repo.update(&entity, esperado).await?;
            let movimientos = repo.count_movimientos(id).await?;
            let hijas = repo.count_hijas(id).await?;
            Ok(CategoriaDetalle::build(&entity, movimientos, hijas))
        }
        .await;
        let detalle = finish_write(tx, outcome).await?;

        info!(id = %detalle.id, "categoría actualizada");
        Ok(detalle)
    }

    pub async fn delete(&self, id: Uuid, row_version: &str) -> AppResult<()> {
        let esperado = parse_row_version(row_version)?;
        let now = self.clock.now_utc();

        let tx = self.uow.begin().await?;
        let outcome = async {
            let repo = tx.categorias();
            repo.find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found(ENTITY, id))?;

            // The foreign keys are `RESTRICT`, so both of these would fail at the database anyway.
            // Asking first turns a constraint violation into a message that names the obstacle.
            let usados = repo.count_movimientos(id).await?;
            if usados > 0 {
                return Err(AppError::DependencyInUse {
                    code: "CATEGORIA_EN_USO",
                    message_key: "Conflict.Categoria.EnUso",
                    params: [("count".to_owned(), usados.to_string())].into(),
                });
            }

            let hijas = repo.count_hijas(id).await?;
            if hijas > 0 {
                return Err(AppError::DependencyInUse {
                    code: "CATEGORIA_CON_HIJAS",
                    message_key: "Conflict.Categoria.ConHijas",
                    params: [("count".to_owned(), hijas.to_string())].into(),
                });
            }

            repo.soft_delete(id, esperado, now).await
        }
        .await;
        finish_write(tx, outcome).await?;

        info!(%id, "categoría eliminada");
        Ok(())
    }
}

async fn load_detalle(repo: &dyn CategoriaRepository, id: Uuid) -> AppResult<CategoriaDetalle> {
    let entity = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::not_found(ENTITY, id))?;
    let movimientos = repo.count_movimientos(id).await?;
    let hijas = repo.count_hijas(id).await?;
    Ok(CategoriaDetalle::build(&entity, movimientos, hijas))
}

async fn ensure_padre_existe(repo: &dyn CategoriaRepository, padre: Option<Uuid>) -> AppResult<()> {
    match padre {
        None => Ok(()),
        Some(padre) if repo.find_by_id(padre).await?.is_some() => Ok(()),
        Some(padre) => Err(AppError::not_found(ENTITY, padre)),
    }
}

/// Rejects `A → B → A` and longer loops. The field validator only sees `A → A`, which is the one
/// case that needs no query.
async fn ensure_sin_ciclo(
    repo: &dyn CategoriaRepository,
    id: Uuid,
    padre: Option<Uuid>,
) -> AppResult<()> {
    let Some(padre) = padre else { return Ok(()) };
    if padre == id || repo.ancestros(padre).await?.contains(&id) {
        return Err(AppError::Validation(vec![crate::FieldError::new(
            "categoriaPadreId",
            "Validation.Categoria.PadreCiclico",
        )]));
    }
    Ok(())
}

/// Uniqueness is per parent: `Materiales` can exist under two different parents, but not twice
/// under the same one, where the user could not tell them apart.
async fn ensure_nombre_libre(
    repo: &dyn CategoriaRepository,
    nombre: &str,
    padre: Option<Uuid>,
    excluir: Option<Uuid>,
) -> AppResult<()> {
    if repo.find_by_nombre(nombre, padre, excluir).await?.is_some() {
        return Err(AppError::Conflict {
            code: "CATEGORIA_NOMBRE_DUPLICADO",
            message_key: "Conflict.Categoria.NombreDuplicado",
            params: [("nombre".to_owned(), nombre.to_owned())].into(),
        });
    }
    Ok(())
}
