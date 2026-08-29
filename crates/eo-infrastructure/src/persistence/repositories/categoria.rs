use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use eo_application::ports::repositories::{
    CategoriaConUso, CategoriaFiltro, CategoriaRepository, SortDir,
};
use eo_application::{AppError, AppResult, PageRequest, PagedResult};
use eo_domain::entities::Categoria;
use eo_domain::{time, RowVersion};
use sea_orm::sea_query::{Alias, Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::categoria as mapper;
use crate::persistence::models::categoria::{self as model, Column, Entity};
use crate::persistence::models::movimiento;

const ENTITY: &str = "Categoria";

/// Guard against a self-referencing chain that never ends, which a corrupt row could otherwise
/// turn into an infinite loop while resolving ancestors.
const MAX_PROFUNDIDAD: usize = 32;

pub struct SeaOrmCategoriaRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmCategoriaRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

#[derive(Debug, FromQueryResult)]
struct RowWithUso {
    id: String,
    nombre: String,
    descripcion: Option<String>,
    color_hex: Option<String>,
    icono: Option<String>,
    categoria_padre_id: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    movimientos_count: i64,
    hijas_count: i64,
    padre_nombre: Option<String>,
}

impl TryFrom<RowWithUso> for CategoriaConUso {
    type Error = AppError;

    fn try_from(row: RowWithUso) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            nombre: row.nombre,
            descripcion: row.descripcion,
            color_hex: row.color_hex,
            icono: row.icono,
            categoria_padre_id: row.categoria_padre_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        Ok(Self {
            categoria: mapper::to_domain(model)?,
            movimientos_count: row.movimientos_count.max(0) as u64,
            hijas_count: row.hijas_count.max(0) as u64,
            padre_nombre: row.padre_nombre,
        })
    }
}

fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

fn filtro_condition(filtro: &CategoriaFiltro) -> Condition {
    let mut condition = alive();
    if let Some(texto) = filtro.texto.as_deref() {
        let patron = format!("%{}%", texto.trim().to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(lower(Column::Nombre).like(patron.clone()))
                .add(lower(Column::Descripcion).like(patron)),
        );
    }
    match filtro.categoria_padre_id {
        None => {}
        Some(None) => condition = condition.add(Column::CategoriaPadreId.is_null()),
        Some(Some(padre)) => {
            condition = condition.add(Column::CategoriaPadreId.eq(padre.to_string()))
        }
    }
    condition
}

/// Correlated count of the live movements of each category, so the listing stays one query.
fn movimientos_count_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col(movimiento::Column::Id).count())
                .from(movimiento::Entity)
                .and_where(
                    Expr::col((movimiento::Entity, movimiento::Column::CategoriaId))
                        .equals((Entity, Column::Id)),
                )
                .and_where(Expr::col((movimiento::Entity, movimiento::Column::IsDeleted)).eq(false))
                .take()
                .into_sub_query_statement(),
        ),
    )
}

fn hijas_count_expr() -> SimpleExpr {
    let hijas = Alias::new("hijas");
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col((hijas.clone(), Column::Id)).count())
                .from_as(Entity, hijas.clone())
                .and_where(
                    Expr::col((hijas.clone(), Column::CategoriaPadreId))
                        .equals((Entity, Column::Id)),
                )
                .and_where(Expr::col((hijas, Column::IsDeleted)).eq(false))
                .take()
                .into_sub_query_statement(),
        ),
    )
}

/// Name of the parent, resolved in the same statement instead of one extra query per row.
fn padre_nombre_expr() -> SimpleExpr {
    let padre = Alias::new("padre");
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col((padre.clone(), Column::Nombre)))
                .from_as(Entity, padre.clone())
                .and_where(
                    Expr::col((padre, Column::Id)).equals((Entity, Column::CategoriaPadreId)),
                )
                .take()
                .into_sub_query_statement(),
        ),
    )
}

#[async_trait]
impl CategoriaRepository for SeaOrmCategoriaRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Categoria>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_by_nombre(
        &self,
        nombre: &str,
        padre: Option<Uuid>,
        excluir: Option<Uuid>,
    ) -> AppResult<Option<Categoria>> {
        let mut condition = alive().add(lower(Column::Nombre).eq(nombre.trim().to_lowercase()));
        condition = match padre {
            Some(padre) => condition.add(Column::CategoriaPadreId.eq(padre.to_string())),
            None => condition.add(Column::CategoriaPadreId.is_null()),
        };
        if let Some(id) = excluir {
            condition = condition.add(Column::Id.ne(id.to_string()));
        }
        let found = Entity::find()
            .filter(condition)
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn search(
        &self,
        filtro: &CategoriaFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<CategoriaConUso>> {
        let condition = filtro_condition(filtro);
        let order = match sort_dir {
            SortDir::Asc => Order::Asc,
            SortDir::Desc => Order::Desc,
        };

        let mut query = Entity::find()
            .filter(condition.clone())
            .column_as(movimientos_count_expr(), "movimientos_count")
            .column_as(hijas_count_expr(), "hijas_count")
            .column_as(padre_nombre_expr(), "padre_nombre");

        query = match sort_by {
            Some("movimientosCount") => {
                query.order_by(Expr::col(Alias::new("movimientos_count")), order)
            }
            Some("hijasCount") => query.order_by(Expr::col(Alias::new("hijas_count")), order),
            Some("createdAt") => query.order_by(Column::CreatedAt, order),
            _ => query.order_by(lower(Column::Nombre), order),
        }
        // Tie-breaker: without it two categories with the same name can swap places between
        // pages, so the second page repeats or skips one.
        .order_by_asc(Column::Id);

        let total = Entity::find()
            .filter(condition)
            .count(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if let Some(limit) = page.limit() {
            query = query.limit(limit).offset(page.offset());
        }

        let rows = query
            .into_model::<RowWithUso>()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let items = rows
            .into_iter()
            .map(CategoriaConUso::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    async fn lookup(&self, texto: Option<&str>, limite: u64) -> AppResult<Vec<Categoria>> {
        let filtro = CategoriaFiltro {
            texto: texto.map(str::to_owned),
            categoria_padre_id: None,
        };
        let rows = Entity::find()
            .filter(filtro_condition(&filtro))
            .order_by_asc(lower(Column::Nombre))
            .limit(limite)
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;
        rows.into_iter().map(mapper::to_domain).collect()
    }

    async fn insert(&self, entity: &Categoria) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Categoria, esperado: RowVersion) -> AppResult<()> {
        let result = Entity::update_many()
            .set(mapper::to_active(entity))
            .filter(Column::Id.eq(entity.id.to_string()))
            .filter(Column::RowVersion.eq(esperado.as_bytes().to_vec()))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if result.rows_affected == 0 {
            return Err(AppError::Concurrency { entity: ENTITY });
        }
        Ok(())
    }

    async fn soft_delete(
        &self,
        id: Uuid,
        esperado: RowVersion,
        at: DateTime<Utc>,
    ) -> AppResult<()> {
        let result = Entity::update_many()
            .col_expr(Column::IsDeleted, Expr::value(true))
            .col_expr(Column::DeletedAt, Expr::value(time::to_storage(at)))
            .col_expr(Column::UpdatedAt, Expr::value(time::to_storage(at)))
            .col_expr(
                Column::RowVersion,
                Expr::value(esperado.next().as_bytes().to_vec()),
            )
            .filter(Column::Id.eq(id.to_string()))
            .filter(Column::RowVersion.eq(esperado.as_bytes().to_vec()))
            .filter(Column::IsDeleted.eq(false))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if result.rows_affected == 0 {
            return Err(AppError::Concurrency { entity: ENTITY });
        }
        Ok(())
    }

    async fn count_movimientos(&self, id: Uuid) -> AppResult<u64> {
        movimiento::Entity::find()
            .filter(movimiento::Column::CategoriaId.eq(id.to_string()))
            .filter(movimiento::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn count_hijas(&self, id: Uuid) -> AppResult<u64> {
        Entity::find()
            .filter(Column::CategoriaPadreId.eq(id.to_string()))
            .filter(alive())
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }

    async fn ancestros(&self, id: Uuid) -> AppResult<Vec<Uuid>> {
        let mut chain = Vec::new();
        let mut actual = Some(id);

        while let Some(current) = actual {
            if chain.len() >= MAX_PROFUNDIDAD || chain.contains(&current) {
                break;
            }
            chain.push(current);
            let Some(row) = self.find_by_id(current).await? else {
                break;
            };
            actual = row.categoria_padre_id;
        }

        // The first element is the starting node, which is not an ancestor of itself.
        Ok(chain.into_iter().skip(1).collect())
    }
}
