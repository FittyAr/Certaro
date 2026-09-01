use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{
    SortDir, TipoMovimientoConUso, TipoMovimientoFiltro, TipoMovimientoRepository,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::TipoMovimiento;
use certaro_domain::{time, RowVersion};
use sea_orm::sea_query::{Alias, Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::tipo_movimiento as mapper;
use crate::persistence::models::movimiento;
use crate::persistence::models::tipo_movimiento::{self as model, Column, Entity};

const ENTITY: &str = "TipoMovimiento";

/// Holds the transaction through an `Arc` rather than a borrow: the unit of work owns both the
/// transaction and the repositories that read from it, and a borrow between two fields of the
/// same struct is a self-reference Rust will not allow.
pub struct SeaOrmTipoMovimientoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmTipoMovimientoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}

/// A row of the listing: the type plus how many live movements use it.
#[derive(Debug, FromQueryResult)]
struct RowWithUso {
    id: String,
    nombre: String,
    descripcion: Option<String>,
    es_ingreso: bool,
    es_sistema: bool,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    movimientos_count: i64,
}

impl TryFrom<RowWithUso> for TipoMovimientoConUso {
    type Error = AppError;

    fn try_from(row: RowWithUso) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            nombre: row.nombre,
            descripcion: row.descripcion,
            es_ingreso: row.es_ingreso,
            es_sistema: row.es_sistema,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        Ok(Self {
            tipo: mapper::to_domain(model)?,
            movimientos_count: row.movimientos_count.max(0) as u64,
        })
    }
}

/// Every read of a business table starts here. Soft-deleted rows are invisible unless a caller
/// deliberately builds its own condition, and no caller does.
fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

fn filtro_condition(filtro: &TipoMovimientoFiltro) -> Condition {
    let mut condition = alive();
    if let Some(texto) = filtro.texto.as_deref() {
        let patron = format!("%{}%", texto.trim().to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(lower(Column::Nombre).like(patron.clone()))
                .add(lower(Column::Descripcion).like(patron)),
        );
    }
    if let Some(es_ingreso) = filtro.es_ingreso {
        condition = condition.add(Column::EsIngreso.eq(es_ingreso));
    }
    if let Some(es_sistema) = filtro.es_sistema {
        condition = condition.add(Column::EsSistema.eq(es_sistema));
    }
    condition
}

/// SQLite's `LIKE` is only case-insensitive for ASCII, so the comparison is lowered explicitly on
/// both sides; otherwise searching «énfasis» would not find «Énfasis».
fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

/// Correlated count of the live movements of each type, so the listing needs one query and not
/// one per row.
fn movimientos_count_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col(movimiento::Column::Id).count())
                .from(movimiento::Entity)
                .and_where(
                    Expr::col((movimiento::Entity, movimiento::Column::TipoMovimientoId))
                        .equals((Entity, Column::Id)),
                )
                .and_where(Expr::col((movimiento::Entity, movimiento::Column::IsDeleted)).eq(false))
                .take()
                .into_sub_query_statement(),
        ),
    )
}

#[async_trait]
impl TipoMovimientoRepository for SeaOrmTipoMovimientoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<TipoMovimiento>> {
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
        excluir: Option<Uuid>,
    ) -> AppResult<Option<TipoMovimiento>> {
        let mut condition = alive().add(lower(Column::Nombre).eq(nombre.trim().to_lowercase()));
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
        filtro: &TipoMovimientoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<TipoMovimientoConUso>> {
        let condition = filtro_condition(filtro);
        let order = match sort_dir {
            SortDir::Asc => Order::Asc,
            SortDir::Desc => Order::Desc,
        };

        let mut query = Entity::find()
            .filter(condition.clone())
            .column_as(movimientos_count_expr(), "movimientos_count");

        query = match sort_by {
            Some("esIngreso") => query.order_by(Column::EsIngreso, order),
            Some("createdAt") => query.order_by(Column::CreatedAt, order),
            Some("movimientosCount") => {
                query.order_by(Expr::col(Alias::new("movimientos_count")), order)
            }
            // The name is both the default and the tie-breaker: an unstable order makes the
            // second page of a listing repeat or skip rows.
            _ => query.order_by(lower(Column::Nombre), order),
        }
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
            .map(TipoMovimientoConUso::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    async fn lookup(&self, texto: Option<&str>, limite: u64) -> AppResult<Vec<TipoMovimiento>> {
        let filtro = TipoMovimientoFiltro {
            texto: texto.map(str::to_owned),
            ..TipoMovimientoFiltro::default()
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

    async fn insert(&self, entity: &TipoMovimiento) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &TipoMovimiento, esperado: RowVersion) -> AppResult<()> {
        // The expected version is part of the `WHERE`, so a concurrent edit loses the race here
        // instead of overwriting what the other user wrote.
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
            .filter(movimiento::Column::TipoMovimientoId.eq(id.to_string()))
            .filter(movimiento::Column::IsDeleted.eq(false))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)
    }
}
