use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use certaro_application::ports::repositories::{
    MovimientoConRelaciones, MovimientoFiltro, MovimientoRepository, MovimientoResumen,
    ReferenciaTabla, SortDir,
};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::Movimiento;
use certaro_domain::{time, Money, RowVersion};
use sea_orm::sea_query::{Alias, Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, DbBackend, EntityTrait, FromQueryResult, JoinType,
    Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Statement,
};
use uuid::Uuid;

use crate::persistence::mappers::movimiento as mapper;
use crate::persistence::models::{
    categoria, cliente, empleado, factura, liquidacion_adelanto, movimiento as model, proyecto,
    tipo_concepto_pago, tipo_movimiento, trabajo,
};

use model::{Column, Entity};

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

/// One row of the listing: the movement plus the names resolved by the joins.
#[derive(Debug, FromQueryResult)]
struct RowConRelaciones {
    id: String,
    fecha: String,
    concepto: String,
    monto: i64,
    cantidad: i64,
    tipo_movimiento_id: String,
    moneda: i32,
    cotizacion_aplicada: Option<i64>,
    tipo_concepto_pago_id: Option<String>,
    categoria_id: Option<String>,
    cliente_id: Option<String>,
    trabajo_id: Option<String>,
    empleado_id: Option<String>,
    factura_id: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    row_version: Vec<u8>,
    is_deleted: bool,
    deleted_at: Option<String>,
    tipo_movimiento_nombre: String,
    es_ingreso: bool,
    categoria_nombre: Option<String>,
    categoria_color: Option<String>,
    cliente_nombre: Option<String>,
    trabajo_descripcion: Option<String>,
    proyecto_nombre: Option<String>,
    adelantos_count: i64,
}

impl TryFrom<RowConRelaciones> for MovimientoConRelaciones {
    type Error = AppError;

    fn try_from(row: RowConRelaciones) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            fecha: row.fecha,
            concepto: row.concepto,
            monto: row.monto,
            cantidad: row.cantidad,
            tipo_movimiento_id: row.tipo_movimiento_id,
            moneda: row.moneda,
            cotizacion_aplicada: row.cotizacion_aplicada,
            tipo_concepto_pago_id: row.tipo_concepto_pago_id,
            categoria_id: row.categoria_id,
            cliente_id: row.cliente_id,
            trabajo_id: row.trabajo_id,
            empleado_id: row.empleado_id,
            factura_id: row.factura_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        Ok(Self {
            movimiento: mapper::to_domain(model)?,
            tipo_movimiento_nombre: row.tipo_movimiento_nombre,
            es_ingreso: row.es_ingreso,
            categoria_nombre: row.categoria_nombre,
            categoria_color: row.categoria_color,
            cliente_nombre: row.cliente_nombre,
            trabajo_descripcion: row.trabajo_descripcion,
            proyecto_nombre: row.proyecto_nombre,
            bloqueado_por_liquidacion: row.adelantos_count > 0,
        })
    }
}

fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

/// A civil date covers the whole day in UTC: `desde` starts at midnight and `hasta` ends at
/// `23:59:59.999`, so a movement booked in the afternoon of the end date is included.
fn desde(date: NaiveDate) -> String {
    time::to_storage(Utc.from_utc_datetime(&date.and_time(NaiveTime::MIN)))
}

fn hasta(date: NaiveDate) -> String {
    let fin = NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap_or(NaiveTime::MIN);
    time::to_storage(Utc.from_utc_datetime(&date.and_time(fin)))
}

fn filtro_condition(filtro: &MovimientoFiltro) -> Condition {
    let mut c = alive();

    if let Some(texto) = filtro.concepto.as_deref() {
        c = c.add(lower(Column::Concepto).like(format!("%{}%", texto.trim().to_lowercase())));
    }
    if let Some(id) = filtro.tipo_movimiento_id {
        c = c.add(Column::TipoMovimientoId.eq(id.to_string()));
    }
    if let Some(id) = filtro.categoria_id {
        c = c.add(Column::CategoriaId.eq(id.to_string()));
    }
    if let Some(id) = filtro.cliente_id {
        c = c.add(Column::ClienteId.eq(id.to_string()));
    }
    if let Some(id) = filtro.trabajo_id {
        c = c.add(Column::TrabajoId.eq(id.to_string()));
    }
    if let Some(id) = filtro.proyecto_id {
        let mut sub = Query::select();
        sub.column(trabajo::Column::Id)
            .from(trabajo::Entity)
            .and_where(Expr::col(trabajo::Column::ProyectoId).eq(id.to_string()))
            .and_where(Expr::col(trabajo::Column::IsDeleted).eq(false));
        c = c.add(Expr::col((Entity, Column::TrabajoId)).in_subquery(sub.take()));
    }
    if let Some(id) = filtro.empleado_id {
        c = c.add(Column::EmpleadoId.eq(id.to_string()));
    }
    if let Some(id) = filtro.factura_id {
        c = c.add(Column::FacturaId.eq(id.to_string()));
    }
    if let Some(moneda) = filtro.moneda {
        c = c.add(Column::Moneda.eq(moneda.as_i32()));
    }
    if let Some(date) = filtro.fecha_desde {
        c = c.add(Column::Fecha.gte(desde(date)));
    }
    if let Some(date) = filtro.fecha_hasta {
        c = c.add(Column::Fecha.lte(hasta(date)));
    }
    // Compared against the unit amount, not the total: that is what the field on screen says.
    if let Some(min) = filtro.monto_min {
        c = c.add(Column::Monto.gte(min.raw()));
    }
    if let Some(max) = filtro.monto_max {
        c = c.add(Column::Monto.lte(max.raw()));
    }
    c
}

/// How many live payroll advances consume this movement. Non-zero means it is frozen.
fn adelantos_count_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col(liquidacion_adelanto::Column::Id).count())
                .from(liquidacion_adelanto::Entity)
                .and_where(
                    Expr::col((
                        liquidacion_adelanto::Entity,
                        liquidacion_adelanto::Column::MovimientoId,
                    ))
                    .equals((Entity, Column::Id)),
                )
                .take()
                .into_sub_query_statement(),
        ),
    )
}

/// The listing query with its joins and its derived columns, shared by `search` and `find_detalle`
/// so a field can never appear in one and not the other.
///
/// The site is reached through the job, which is the only way a movement is charged to one.
fn base_query() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, model::Relation::TipoMovimiento.def())
        .join(JoinType::LeftJoin, model::Relation::Categoria.def())
        .join(JoinType::LeftJoin, model::Relation::Cliente.def())
        .join(JoinType::LeftJoin, model::Relation::Trabajo.def())
        .join(JoinType::LeftJoin, trabajo::Relation::Proyecto.def())
        .column_as(
            Expr::col((cliente::Entity, cliente::Column::Nombre)),
            "cliente_nombre",
        )
        .column_as(
            Expr::col((trabajo::Entity, trabajo::Column::Descripcion)),
            "trabajo_descripcion",
        )
        .column_as(
            Expr::col((proyecto::Entity, proyecto::Column::Nombre)),
            "proyecto_nombre",
        )
        .column_as(
            Expr::col((tipo_movimiento::Entity, tipo_movimiento::Column::Nombre)),
            "tipo_movimiento_nombre",
        )
        .column_as(
            Expr::col((tipo_movimiento::Entity, tipo_movimiento::Column::EsIngreso)),
            "es_ingreso",
        )
        .column_as(
            Expr::col((categoria::Entity, categoria::Column::Nombre)),
            "categoria_nombre",
        )
        .column_as(
            Expr::col((categoria::Entity, categoria::Column::ColorHex)),
            "categoria_color",
        )
        .column_as(adelantos_count_expr(), "adelantos_count")
}

#[derive(Debug, FromQueryResult)]
struct ResumenRow {
    es_ingreso: bool,
    suma_bruta: Option<i64>,
    cantidad: i64,
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
        let (sql, values) = Query::select()
            .expr_as(
                Expr::col((tipo_movimiento::Entity, tipo_movimiento::Column::EsIngreso)),
                Alias::new("es_ingreso"),
            )
            .expr_as(
                SimpleExpr::from(Func::sum(
                    Expr::col((Entity, Column::Monto)).mul(Expr::col((Entity, Column::Cantidad))),
                )),
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
