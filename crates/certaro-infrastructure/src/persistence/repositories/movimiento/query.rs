use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use certaro_application::ports::repositories::{MovimientoConRelaciones, MovimientoFiltro};
use certaro_application::AppError;
use certaro_domain::time;
use sea_orm::sea_query::{Expr, Func, Query, SimpleExpr};
use sea_orm::{ColumnTrait, Condition, EntityTrait, FromQueryResult, JoinType, QuerySelect, RelationTrait};
use crate::persistence::mappers::movimiento as mapper;
use crate::persistence::models::{
    categoria, cliente, liquidacion_adelanto, movimiento as model, proyecto,
    tipo_movimiento, trabajo,
};
use model::{Column, Entity};

#[derive(Debug, FromQueryResult)]
pub(super) struct RowConRelaciones {
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

pub(super) fn alive() -> Condition {
    Condition::all().add(Column::IsDeleted.eq(false))
}

pub(super) fn lower(column: Column) -> SimpleExpr {
    Func::lower(Expr::col((Entity, column))).into()
}

/// A civil date covers the whole day in UTC: `desde` starts at midnight and `hasta` ends at
/// `23:59:59.999`, so a movement booked in the afternoon of the end date is included.
pub(super) fn desde(date: NaiveDate) -> String {
    time::to_storage(Utc.from_utc_datetime(&date.and_time(NaiveTime::MIN)))
}

pub(super) fn hasta(date: NaiveDate) -> String {
    let fin = NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap_or(NaiveTime::MIN);
    time::to_storage(Utc.from_utc_datetime(&date.and_time(fin)))
}

pub(super) fn filtro_condition(filtro: &MovimientoFiltro) -> Condition {
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
pub(super) fn adelantos_count_expr() -> SimpleExpr {
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
pub(super) fn base_query() -> sea_orm::Select<Entity> {
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
pub(super) struct ResumenRow {
    pub(super) es_ingreso: bool,
    pub(super) suma_bruta: Option<i64>,
    pub(super) cantidad: i64,
}
