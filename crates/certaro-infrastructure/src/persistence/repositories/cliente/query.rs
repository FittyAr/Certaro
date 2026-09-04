use certaro_application::ports::repositories::ClienteConResumen;
use certaro_application::{AppError, AppResult};
use certaro_domain::entities::Cliente;
use certaro_domain::Money;
use sea_orm::sea_query::{Alias, Expr, ExprTrait, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::cliente as mapper;
use crate::persistence::models::cliente::{self as model, Column, Entity};
use crate::persistence::models::{factura, movimiento, pago_factura, proyecto};

use super::super::estado_deuda_ids;
use certaro_application::ports::repositories::{ClienteFiltro, SortDir};
use certaro_application::{PageRequest, PagedResult};

#[derive(Debug, FromQueryResult)]
pub(crate) struct RowConResumen {
    pub id: String,
    pub nombre: String,
    pub cuit: Option<String>,
    pub direccion: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub condicion_iva: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    pub proyectos_count: i64,
    pub facturas_count: i64,
    pub deuda: i64,
}

impl TryFrom<RowConResumen> for ClienteConResumen {
    type Error = AppError;

    fn try_from(row: RowConResumen) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            nombre: row.nombre,
            cuit: row.cuit,
            direccion: row.direccion,
            telefono: row.telefono,
            email: row.email,
            condicion_iva: row.condicion_iva,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        Ok(Self {
            cliente: mapper::to_domain(model)?,
            proyectos_count: row.proyectos_count.max(0) as u64,
            facturas_count: row.facturas_count.max(0) as u64,
            deuda: Money::from_raw(row.deuda),
        })
    }
}

pub(crate) fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

pub(crate) fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

pub(crate) fn filtro_condition(filtro: &ClienteFiltro) -> Condition {
    let mut c = alive();
    if let Some(texto) = filtro.texto.as_deref() {
        let patron = format!("%{}%", texto.trim().to_lowercase());
        c = c.add(
            Condition::any()
                .add(lower(Column::Nombre).like(patron.clone()))
                .add(lower(Column::Cuit).like(patron.clone()))
                .add(lower(Column::Email).like(patron)),
        );
    }
    if let Some(condicion) = filtro.condicion_iva.as_deref() {
        c = c.add(Column::CondicionIva.eq(condicion));
    }
    if filtro.solo_con_deuda {
        c = c.add(deuda_expr().gt(0i64));
    }
    c
}

pub(crate) fn proyectos_count_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col(proyecto::Column::Id).count())
                .from(proyecto::Entity)
                .and_where(
                    Expr::col((proyecto::Entity, proyecto::Column::ClienteId)).equals((Entity, Column::Id)),
                )
                .and_where(Expr::col((proyecto::Entity, proyecto::Column::IsDeleted)).eq(false))
                .take()
                .into_sub_query_statement(),
        ),
    )
}

pub(crate) fn facturas_count_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col(factura::Column::Id).count())
                .from(factura::Entity)
                .and_where(
                    Expr::col((factura::Entity, factura::Column::ClienteId))
                        .equals((Entity, Column::Id)),
                )
                .and_where(Expr::col((factura::Entity, factura::Column::IsDeleted)).eq(false))
                .take()
                .into_sub_query_statement(),
        ),
    )
}

pub(crate) fn deuda_expr() -> SimpleExpr {
    let pagado = SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Func::coalesce([
                    Func::sum(Expr::col(pago_factura::Column::Monto)).into(),
                    Expr::value(0i64),
                ]))
                .from(pago_factura::Entity)
                .and_where(
                    Expr::col((pago_factura::Entity, pago_factura::Column::FacturaId))
                        .equals((factura::Entity, factura::Column::Id)),
                )
                .and_where(
                    Expr::col((pago_factura::Entity, pago_factura::Column::IsDeleted)).eq(false),
                )
                .take()
                .into_sub_query_statement(),
        ),
    );

    Func::coalesce([
        SimpleExpr::SubQuery(
            None,
            Box::new(
                Query::select()
                    .expr(Func::sum(
                        Expr::col((factura::Entity, factura::Column::Total)).sub(pagado),
                    ))
                    .from(factura::Entity)
                    .and_where(
                        Expr::col((factura::Entity, factura::Column::ClienteId))
                            .equals((Entity, Column::Id)),
                    )
                    .and_where(Expr::col((factura::Entity, factura::Column::IsDeleted)).eq(false))
                    .and_where(
                        Expr::col((factura::Entity, factura::Column::Estado))
                            .is_in(estado_deuda_ids()),
                    )
                    .take()
                    .into_sub_query_statement(),
            ),
        ),
        Expr::value(0i64),
    ])
    .into()
}

pub(crate) async fn search_clientes(
    conn: &DatabaseTransaction,
    filtro: &ClienteFiltro,
    page: PageRequest,
    sort_by: Option<&str>,
    sort_dir: SortDir,
) -> AppResult<PagedResult<ClienteConResumen>> {
    let condition = filtro_condition(filtro);
    let order = match sort_dir {
        SortDir::Asc => Order::Asc,
        SortDir::Desc => Order::Desc,
    };

    let mut query = Entity::find()
        .filter(condition.clone())
        .column_as(proyectos_count_expr(), "proyectos_count")
        .column_as(facturas_count_expr(), "facturas_count")
        .column_as(deuda_expr(), "deuda");

    query = match sort_by {
        Some("cuit") => query.order_by(lower(Column::Cuit), order),
        Some("deuda") => query.order_by(Expr::col(Alias::new("deuda")), order),
        Some("proyectosCount") => query.order_by(Expr::col(Alias::new("proyectos_count")), order),
        Some("facturasCount") => query.order_by(Expr::col(Alias::new("facturas_count")), order),
        Some("createdAt") => query.order_by(Column::CreatedAt, order),
        _ => query.order_by(lower(Column::Nombre), order),
    }
    .order_by_asc(Column::Id);

    let total = Entity::find()
        .filter(condition)
        .count(conn)
        .await
        .map_err(AppError::persistence)?;

    if let Some(limit) = page.limit() {
        query = query.limit(limit).offset(page.offset());
    }

    let rows = query
        .into_model::<RowConResumen>()
        .all(conn)
        .await
        .map_err(AppError::persistence)?;

    let items = rows
        .into_iter()
        .map(ClienteConResumen::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PagedResult::new(items, total, page))
}

pub(crate) async fn lookup_clientes(
    conn: &DatabaseTransaction,
    texto: Option<&str>,
    limite: u64,
) -> AppResult<Vec<Cliente>> {
    let filtro = ClienteFiltro {
        texto: texto.map(str::to_owned),
        ..ClienteFiltro::default()
    };
    let rows = Entity::find()
        .filter(filtro_condition(&filtro))
        .order_by_asc(lower(Column::Nombre))
        .limit(limite)
        .all(conn)
        .await
        .map_err(AppError::persistence)?;
    rows.into_iter().map(mapper::to_domain).collect()
}

pub(crate) async fn count_proyectos(conn: &DatabaseTransaction, id: Uuid) -> AppResult<u64> {
    proyecto::Entity::find()
        .filter(proyecto::Column::ClienteId.eq(id.to_string()))
        .filter(proyecto::Column::IsDeleted.eq(false))
        .count(conn)
        .await
        .map_err(AppError::persistence)
}

pub(crate) async fn count_facturas(conn: &DatabaseTransaction, id: Uuid) -> AppResult<u64> {
    factura::Entity::find()
        .filter(factura::Column::ClienteId.eq(id.to_string()))
        .filter(factura::Column::IsDeleted.eq(false))
        .count(conn)
        .await
        .map_err(AppError::persistence)
}

pub(crate) async fn count_movimientos(conn: &DatabaseTransaction, id: Uuid) -> AppResult<u64> {
    movimiento::Entity::find()
        .filter(movimiento::Column::ClienteId.eq(id.to_string()))
        .filter(movimiento::Column::IsDeleted.eq(false))
        .count(conn)
        .await
        .map_err(AppError::persistence)
}
