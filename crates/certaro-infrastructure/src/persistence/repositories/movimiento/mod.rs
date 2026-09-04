use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use certaro_application::ports::repositories::{
    MovimientoConRelaciones, MovimientoFiltro, MovimientoRepository, MovimientoResumen,
    ReferenciaTabla, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::Movimiento;
use certaro_domain::{time, Money, RowVersion};
use sea_orm::sea_query::{Alias, Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, DbBackend, EntityTrait, FromQueryResult, Order, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Statement,
};
use uuid::Uuid;

use crate::persistence::mappers::movimiento as mapper;
use crate::persistence::models::{
    categoria, cliente, empleado, factura, liquidacion_adelanto, movimiento as model,
    tipo_concepto_pago, tipo_movimiento, trabajo,
};
use model::{Column, Entity};

mod query;
use query::*;

const ENTITY: &str = "Movimiento";

pub struct SeaOrmMovimientoRepository {
    tx: Arc<DatabaseTransaction>,
}

impl SeaOrmMovimientoRepository {
    pub fn new(tx: Arc<DatabaseTransaction>) -> Self {
        Self { tx }
    }

    fn conn(&self) -> &DatabaseTransaction {
        self.tx.as_ref()
    }
}


#[async_trait]
impl MovimientoRepository for SeaOrmMovimientoRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Movimiento>> {
        let found = Entity::find_by_id(id.to_string())
            .filter(alive())
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(mapper::to_domain).transpose()
    }

    async fn find_detalle(&self, id: Uuid) -> AppResult<Option<MovimientoConRelaciones>> {
        let found = base_query()
            .filter(alive())
            .filter(Column::Id.eq(id.to_string()))
            .into_model::<RowConRelaciones>()
            .one(self.conn())
            .await
            .map_err(AppError::persistence)?;
        found.map(MovimientoConRelaciones::try_from).transpose()
    }

    async fn search(
        &self,
        filtro: &MovimientoFiltro,
        page: PageRequest,
        sort_by: Option<&str>,
        sort_dir: SortDir,
    ) -> AppResult<PagedResult<MovimientoConRelaciones>> {
        let condition = filtro_condition(filtro);
        // The default is newest first: the screen is a ledger, and the last thing entered is what
        // the user is looking for.
        let order = match (sort_by, sort_dir) {
            (None, SortDir::Asc) => Order::Desc,
            (_, SortDir::Asc) => Order::Asc,
            (_, SortDir::Desc) => Order::Desc,
        };

        let mut query = base_query().filter(condition.clone());

        query = match sort_by {
            Some("concepto") => query.order_by(lower(Column::Concepto), order),
            Some("monto") => query.order_by(Column::Monto, order),
            // The total is not stored, so it is ordered by the product itself.
            Some("total") => query.order_by(
                Expr::col((Entity, Column::Monto)).mul(Expr::col((Entity, Column::Cantidad))),
                order,
            ),
            Some("tipoMovimientoNombre") => query.order_by(
                SimpleExpr::from(Func::lower(Expr::col((
                    tipo_movimiento::Entity,
                    tipo_movimiento::Column::Nombre,
                )))),
                order,
            ),
            Some("categoriaNombre") => query.order_by(
                SimpleExpr::from(Func::lower(Expr::col((
                    categoria::Entity,
                    categoria::Column::Nombre,
                )))),
                order,
            ),
            _ => query.order_by(Column::Fecha, order),
        }
        // Tie-breaker: two movements booked at the same instant have no inherent order, and
        // without this the paging repeats or skips rows between requests.
        .order_by_desc(Column::Id);

        let total = Entity::find()
            .filter(condition)
            .count(self.conn())
            .await
            .map_err(AppError::persistence)?;

        if let Some(limit) = page.limit() {
            query = query.limit(limit).offset(page.offset());
        }

        let rows = query
            .into_model::<RowConRelaciones>()
            .all(self.conn())
            .await
            .map_err(AppError::persistence)?;

        let items = rows
            .into_iter()
            .map(MovimientoConRelaciones::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PagedResult::new(items, total, page))
    }

    async fn resumen(&self, filtro: &MovimientoFiltro) -> AppResult<MovimientoResumen> {
        // The sign lives in the type, so the aggregation groups by it and the two buckets come
        // back in one query rather than two.
        let monto_expr = if filtro.moneda.is_none() {
            Expr::cust(
                "CASE WHEN movimientos.moneda = 1 AND movimientos.cotizacion_aplicada IS NOT NULL AND movimientos.cotizacion_aplicada > 0 \
                 THEN (movimientos.monto * movimientos.cotizacion_aplicada / 10000) * movimientos.cantidad \
                 ELSE movimientos.monto * movimientos.cantidad END",
            )
        } else {
            Expr::col((Entity, Column::Monto)).mul(Expr::col((Entity, Column::Cantidad)))
        };

        let (sql, values) = Query::select()
            .expr_as(
                Expr::col((tipo_movimiento::Entity, tipo_movimiento::Column::EsIngreso)),
                Alias::new("es_ingreso"),
            )
            .expr_as(
                SimpleExpr::from(Func::sum(monto_expr)),
                Alias::new("suma_bruta"),
            )
            .expr_as(
                Expr::col((Entity, Column::Id)).count(),
                Alias::new("cantidad"),
            )
            .from(Entity)
            .inner_join(
                tipo_movimiento::Entity,
                Expr::col((tipo_movimiento::Entity, tipo_movimiento::Column::Id))
                    .equals((Entity, Column::TipoMovimientoId)),
            )
            .cond_where(filtro_condition(filtro))
            .add_group_by([Expr::col((
                tipo_movimiento::Entity,
                tipo_movimiento::Column::EsIngreso,
            ))
            .into()])
            .build(sea_orm::sea_query::SqliteQueryBuilder);

        let rows = ResumenRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            sql,
            values,
        ))
        .all(self.conn())
        .await
        .map_err(AppError::persistence)?;

        let mut ingresos = Money::ZERO;
        let mut gastos = Money::ZERO;
        let mut cantidad: u64 = 0;

        for row in rows {
            // The product of two values scaled by 10 000 is scaled by 100 000 000, so the sum
            // comes back at that scale and is narrowed here, not in SQL.
            let monto = Money::from_product_sum(i128::from(row.suma_bruta.unwrap_or(0)))
                .map_err(AppError::from)?;
            if row.es_ingreso {
                ingresos = monto;
            } else {
                gastos = monto;
            }
            cantidad += row.cantidad.max(0) as u64;
        }

        Ok(MovimientoResumen {
            total_ingresos: ingresos,
            total_gastos: gastos,
            balance: ingresos.checked_sub(gastos).map_err(AppError::from)?,
            cantidad,
        })
    }

    async fn insert(&self, entity: &Movimiento) -> AppResult<()> {
        Entity::insert(mapper::to_active(entity))
            .exec(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(())
    }

    async fn update(&self, entity: &Movimiento, esperado: RowVersion) -> AppResult<()> {
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

    async fn esta_en_liquidacion(&self, id: Uuid) -> AppResult<bool> {
        let count = liquidacion_adelanto::Entity::find()
            .filter(liquidacion_adelanto::Column::MovimientoId.eq(id.to_string()))
            .count(self.conn())
            .await
            .map_err(AppError::persistence)?;
        Ok(count > 0)
    }

    async fn existe_referencia(&self, tabla: ReferenciaTabla, id: Uuid) -> AppResult<bool> {
        let id = id.to_string();
        // Each arm names its table through the typed entity, so no table name is ever built from
        // a string.
        let count = match tabla {
            ReferenciaTabla::TipoMovimiento => {
                tipo_movimiento::Entity::find()
                    .filter(tipo_movimiento::Column::Id.eq(id))
                    .filter(tipo_movimiento::Column::IsDeleted.eq(false))
                    .count(self.conn())
                    .await
            }
            ReferenciaTabla::TipoConceptoPago => {
                tipo_concepto_pago::Entity::find()
                    .filter(tipo_concepto_pago::Column::Id.eq(id))
                    .filter(tipo_concepto_pago::Column::IsDeleted.eq(false))
                    .count(self.conn())
                    .await
            }
            ReferenciaTabla::Categoria => {
                categoria::Entity::find()
                    .filter(categoria::Column::Id.eq(id))
                    .filter(categoria::Column::IsDeleted.eq(false))
                    .count(self.conn())
                    .await
            }
            ReferenciaTabla::Cliente => {
                cliente::Entity::find()
                    .filter(cliente::Column::Id.eq(id))
                    .filter(cliente::Column::IsDeleted.eq(false))
                    .count(self.conn())
                    .await
            }
            ReferenciaTabla::Trabajo => {
                trabajo::Entity::find()
                    .filter(trabajo::Column::Id.eq(id))
                    .filter(trabajo::Column::IsDeleted.eq(false))
                    .count(self.conn())
                    .await
            }
            ReferenciaTabla::Empleado => {
                empleado::Entity::find()
                    .filter(empleado::Column::Id.eq(id))
                    .filter(empleado::Column::IsDeleted.eq(false))
                    .count(self.conn())
                    .await
            }
            ReferenciaTabla::Factura => {
                factura::Entity::find()
                    .filter(factura::Column::Id.eq(id))
                    .filter(factura::Column::IsDeleted.eq(false))
                    .count(self.conn())
                    .await
            }
        };
        Ok(count.map_err(AppError::persistence)? > 0)
    }
}

