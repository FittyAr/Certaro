use certaro_application::ports::repositories::{ProyectoConResumen, ProyectoFiltro, SortDir};
use certaro_application::{AppError, AppResult, PageRequest, PagedResult};
use certaro_domain::entities::Proyecto;
use certaro_domain::{EstadoProyecto, Money};
use sea_orm::sea_query::{Alias, Expr, Func, Query, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult, JoinType, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::persistence::mappers::proyecto as mapper;
use crate::persistence::models::proyecto::{self as model, Column, Entity};
use crate::persistence::models::{cliente, movimiento, tipo_movimiento, trabajo};

#[derive(Debug, FromQueryResult)]
pub(crate) struct RowConResumen {
    pub id: String,
    pub numero: i32,
    pub nombre: String,
    pub direccion: Option<String>,
    pub localidad: Option<String>,
    pub cliente_id: String,
    pub estado: i32,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub row_version: Vec<u8>,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    pub cliente_nombre: String,
    pub trabajos_count: i64,
    pub rentabilidad: i64,
}

impl TryFrom<RowConResumen> for ProyectoConResumen {
    type Error = AppError;

    fn try_from(row: RowConResumen) -> Result<Self, Self::Error> {
        let model = model::Model {
            id: row.id,
            numero: row.numero,
            nombre: row.nombre,
            direccion: row.direccion,
            localidad: row.localidad,
            cliente_id: row.cliente_id,
            estado: row.estado,
            created_at: row.created_at,
            updated_at: row.updated_at,
            row_version: row.row_version,
            is_deleted: row.is_deleted,
            deleted_at: row.deleted_at,
        };
        Ok(Self {
            proyecto: mapper::to_domain(model)?,
            cliente_nombre: row.cliente_nombre,
            trabajos_count: row.trabajos_count.max(0) as u64,
            // The subquery already sums the product of two scaled values, so it comes back at
            // scale 10^8 and is narrowed here.
            rentabilidad: Money::from_product_sum(i128::from(row.rentabilidad))
                .map_err(AppError::from)?,
        })
    }
}

pub(crate) fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

pub(crate) fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

pub(crate) fn cliente_join() -> sea_orm::RelationDef {
    Entity::belongs_to(cliente::Entity)
        .from(Column::ClienteId)
        .to(cliente::Column::Id)
        .into()
}

pub(crate) fn filtro_condition(filtro: &ProyectoFiltro) -> Condition {
    let mut c = alive();
    if let Some(texto) = filtro.texto.as_deref() {
        let limpio = texto.trim();
        let patron = format!("%{}%", limpio.to_lowercase());
        let mut any = Condition::any()
            .add(lower(Column::Nombre).like(patron.clone()))
            .add(lower(Column::Direccion).like(patron.clone()))
            .add(lower(Column::Localidad).like(patron));
        if let Ok(numero) = limpio.parse::<i32>() {
            any = any.add(Column::Numero.eq(numero));
        }
        c = c.add(any);
    }
    if let Some(id) = filtro.cliente_id {
        c = c.add(Column::ClienteId.eq(id.to_string()));
    }
    if let Some(estado) = filtro.estado {
        c = c.add(Column::Estado.eq(estado.as_i32()));
    }
    if filtro.solo_activas {
        c = c
            .add(Column::Estado.is_in([EstadoProyecto::Activa.as_i32(), EstadoProyecto::Pausada.as_i32()]));
    }
    c
}

pub(crate) fn trabajos_count_expr() -> SimpleExpr {
    SimpleExpr::SubQuery(
        None,
        Box::new(
            Query::select()
                .expr(Expr::col(trabajo::Column::Id).count())
                .from(trabajo::Entity)
                .and_where(
                    Expr::col((trabajo::Entity, trabajo::Column::ProyectoId))
                        .equals((Entity, Column::Id)),
                )
                .and_where(Expr::col((trabajo::Entity, trabajo::Column::IsDeleted)).eq(false))
                .take()
                .into_sub_query_statement(),
        ),
    )
}

pub(crate) fn suma_movimientos_expr(es_ingreso: bool) -> SimpleExpr {
    Func::coalesce([
        SimpleExpr::SubQuery(
            None,
            Box::new(
                Query::select()
                    .expr(Func::sum(
                        Expr::col((movimiento::Entity, movimiento::Column::Monto)).mul(Expr::col(
                            (movimiento::Entity, movimiento::Column::Cantidad),
                        )),
                    ))
                    .from(movimiento::Entity)
                    .inner_join(
                        trabajo::Entity,
                        Expr::col((trabajo::Entity, trabajo::Column::Id))
                            .equals((movimiento::Entity, movimiento::Column::TrabajoId)),
                    )
                    .inner_join(
                        tipo_movimiento::Entity,
                        Expr::col((tipo_movimiento::Entity, tipo_movimiento::Column::Id))
                            .equals((movimiento::Entity, movimiento::Column::TipoMovimientoId)),
                    )
                    .and_where(
                        Expr::col((trabajo::Entity, trabajo::Column::ProyectoId))
                            .equals((Entity, Column::Id)),
                    )
                    .and_where(
                        Expr::col((tipo_movimiento::Entity, tipo_movimiento::Column::EsIngreso))
                            .eq(es_ingreso),
                    )
                    .and_where(
                        Expr::col((movimiento::Entity, movimiento::Column::IsDeleted)).eq(false),
                    )
                    .and_where(Expr::col((trabajo::Entity, trabajo::Column::IsDeleted)).eq(false))
                    .take()
                    .into_sub_query_statement(),
            ),
        ),
        Expr::value(0i64),
    ])
    .into()
}

pub(crate) fn rentabilidad_expr() -> SimpleExpr {
    suma_movimientos_expr(true).sub(suma_movimientos_expr(false))
}

pub(crate) fn base_query() -> sea_orm::Select<Entity> {
    Entity::find()
        .join(JoinType::InnerJoin, cliente_join())
        .column_as(
            Expr::col((cliente::Entity, cliente::Column::Nombre)),
            "cliente_nombre",
        )
        .column_as(trabajos_count_expr(), "trabajos_count")
        .column_as(rentabilidad_expr(), "rentabilidad")
}

pub(crate) async fn search_proyectos(
    conn: &DatabaseTransaction,
    filtro: &ProyectoFiltro,
    page: PageRequest,
    sort_by: Option<&str>,
    sort_dir: SortDir,
) -> AppResult<PagedResult<ProyectoConResumen>> {
    let condition = filtro_condition(filtro);
    let order = match sort_dir {
        SortDir::Asc => Order::Asc,
        SortDir::Desc => Order::Desc,
    };

    let mut query = base_query().filter(condition.clone());

    query = match sort_by {
        Some("nombre") => query.order_by(lower(Column::Nombre), order),
        Some("estado") => query.order_by(Column::Estado, order),
        Some("clienteNombre") => query.order_by(
            SimpleExpr::from(Func::lower(Expr::col((
                cliente::Entity,
                cliente::Column::Nombre,
            )))),
            order,
        ),
        Some("trabajosCount") => query.order_by(Expr::col(Alias::new("trabajos_count")), order),
        Some("rentabilidad") => query.order_by(Expr::col(Alias::new("rentabilidad")), order),
        Some("createdAt") => query.order_by(Column::CreatedAt, order),
        _ => query.order_by(
            Column::Numero,
            match sort_by {
                None => Order::Desc,
                Some(_) => order,
            },
        ),
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
        .map(ProyectoConResumen::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PagedResult::new(items, total, page))
}

pub(crate) async fn lookup_proyectos(
    conn: &DatabaseTransaction,
    cliente_id: Option<Uuid>,
    texto: Option<&str>,
    limite: u64,
) -> AppResult<Vec<Proyecto>> {
    let filtro = ProyectoFiltro {
        texto: texto.map(str::to_owned),
        cliente_id,
        ..ProyectoFiltro::default()
    };
    let rows = Entity::find()
        .filter(filtro_condition(&filtro))
        .order_by_desc(Column::Numero)
        .limit(limite)
        .all(conn)
        .await
        .map_err(AppError::persistence)?;
    rows.into_iter().map(mapper::to_domain).collect()
}
